//! UI-independent device identities, command policies, and parsing contracts.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Device transport. A serial may be observed on different transports over time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    Adb,
    Fastboot,
}

/// An observed device; selection is always a separate, explicit UI action.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub serial: String,
    pub model: String,
    pub state: String,
    pub transport: Transport,
}

impl Device {
    /// Whether normal Android commands can currently be issued.
    pub fn online(&self) -> bool {
        self.transport == Transport::Adb && self.state == "device"
    }
    /// Stable transport-aware identity for a selection.
    pub fn key(&self) -> String {
        format!("{:?}:{}", self.transport, self.serial)
    }
}

/// Device details shown by the dashboard.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceInfo {
    pub model: String,
    pub android: String,
    pub battery: String,
    pub charging: String,
}

/// An entry in the Android filesystem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub directory: bool,
    pub size: u64,
}

/// Host process invocation. Arguments never pass through a host shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub timeout: Duration,
    /// Per-child environment overrides; never mutate the parent environment.
    pub env: Vec<(String, String)>,
}

impl CommandSpec {
    /// Construct a bounded invocation without interpreting arguments.
    pub fn new(program: &str, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            timeout: Duration::from_secs(30),
            env: Vec::new(),
        }
    }
}

/// Typed service failures. Raw process output is for the user, never telemetry.
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("{0}")]
    Invalid(String),
    #[error("Could not start {0}. Install it and check PATH: {1}")]
    Spawn(String, String),
    #[error("Operation failed: {0}")]
    Failed(String),
    #[error("Operation timed out; device-side changes may be incomplete")]
    Timeout,
    #[error("Operation cancelled; device-side changes may be incomplete")]
    Cancelled,
    #[error("Command output exceeded the 4 MiB limit")]
    OutputLimit,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Parse discovery output while retaining unauthorized/offline/recovery devices.
pub fn parse_devices(text: &str, transport: Transport) -> Result<Vec<Device>, DeviceError> {
    let mut devices = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if line.starts_with("List of devices") || line.starts_with('*') {
            continue;
        }
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(DeviceError::Invalid(
                "Malformed device discovery output".into(),
            ));
        }
        let model = parts
            .iter()
            .find_map(|p| p.strip_prefix("model:"))
            .unwrap_or("Android device");
        devices.push(Device {
            serial: parts[0].into(),
            model: model.replace('_', " "),
            state: if transport == Transport::Fastboot {
                "fastboot".into()
            } else {
                parts[1].into()
            },
            transport: transport.clone(),
        });
    }
    Ok(devices)
}

/// Quote one argument for Android's POSIX shell, independently of host arguments.
pub fn remote_quote(value: &str) -> Result<String, DeviceError> {
    if value.chars().any(|c| c == '\0' || c == '\n' || c == '\r') {
        return Err(DeviceError::Invalid(
            "Paths and arguments cannot contain NUL or line breaks".into(),
        ));
    }
    Ok(format!("'{}'", value.replace('\'', "'\\''")))
}

/// Validate an absolute Android path and protect shared-storage roots on mutation.
pub fn validate_path(path: &str, mutation: bool) -> Result<(), DeviceError> {
    if !path.starts_with('/') || path.split('/').any(|p| p == ".." || p == ".") {
        return Err(DeviceError::Invalid(
            "Use an absolute Android path without traversal".into(),
        ));
    }
    remote_quote(path)?;
    if mutation
        && !["/sdcard/", "/storage/emulated/0/"]
            .iter()
            .any(|root| path.starts_with(root) && !path[root.len()..].trim_matches('/').is_empty())
    {
        return Err(DeviceError::Invalid(
            "Changes are limited to items inside shared storage, never its root".into(),
        ));
    }
    Ok(())
}

/// Validate package IDs before embedding them in a managed device command.
pub fn validate_package(package: &str) -> Result<(), DeviceError> {
    if package.is_empty()
        || package.split('.').any(str::is_empty)
        || !package
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(DeviceError::Invalid(
            "Invalid Android package identifier".into(),
        ));
    }
    Ok(())
}

/// Parse NUL-framed stat/name records and require direct-child membership.
/// Newline filenames are rejected rather than allowed to forge actionable rows.
pub fn parse_files(text: &str, directory: &str) -> Result<Vec<FileEntry>, DeviceError> {
    validate_path(directory, false)?;
    if !text.is_empty() && !text.ends_with('\0') {
        return Err(DeviceError::Invalid("Incomplete file listing".into()));
    }
    let parent = directory.trim_end_matches('/');
    let mut entries = Vec::new();
    for record in text.split_terminator('\0') {
        let (metadata, path) = record
            .split_once('\n')
            .ok_or_else(|| DeviceError::Invalid("Malformed file record".into()))?;
        let (kind, size) = metadata
            .split_once('|')
            .ok_or_else(|| DeviceError::Invalid("Malformed file metadata".into()))?;
        validate_path(path, false)?;
        if path.rsplit_once('/').map(|(p, _)| p.trim_end_matches('/')) != Some(parent) {
            return Err(DeviceError::Invalid(
                "File listing returned an item outside the requested directory".into(),
            ));
        }
        entries.push(FileEntry {
            path: path.into(),
            name: path.rsplit('/').next().unwrap_or(path).into(),
            directory: kind == "directory",
            size: size
                .parse()
                .map_err(|_| DeviceError::Invalid("Invalid file size".into()))?,
        });
    }
    entries.sort_by(|a, b| {
        b.directory
            .cmp(&a.directory)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_preserves_states_and_transport() {
        let devices = parse_devices(
            "List of devices attached\na unauthorized\nb device model:Pixel_8\n",
            Transport::Adb,
        )
        .unwrap();
        assert_eq!(devices[0].state, "unauthorized");
        assert!(!devices[0].online());
        assert_eq!(devices[1].model, "Pixel 8");
        assert!(devices[1].online());
        let boot = parse_devices("b\tfastboot\n", Transport::Fastboot).unwrap();
        assert_ne!(devices[1].key(), boot[0].key());
        assert!(parse_devices("garbage", Transport::Adb).is_err());
    }

    #[test]
    fn managed_paths_reject_roots_traversal_and_controls() {
        for path in [
            "/",
            "/sdcard/",
            "/sdcard//",
            "/storage/emulated/0",
            "/sdcard/../system",
            "-rf",
            "/sdcard/a\nb",
            "/sdcard/./a",
        ] {
            assert!(validate_path(path, true).is_err(), "{path}");
        }
        assert!(validate_path("/sdcard/Camera/a b'λ.jpg", true).is_ok());
        assert!(validate_path("/system", false).is_ok());
    }

    #[test]
    fn quoting_does_not_interpret_shell_input() {
        assert_eq!(remote_quote("a'b; $(id)").unwrap(), "'a'\\''b; $(id)'");
        assert!(remote_quote("a\0b").is_err());
    }

    #[test]
    fn package_ids_cannot_inject_options_or_commands() {
        for package in ["", "-r", "com.a;id", "com..a", "com.a b"] {
            assert!(validate_package(package).is_err());
        }
        assert!(validate_package("com.example_app.app2").is_ok());
    }

    #[test]
    fn file_records_preserve_spaces_and_pipes_and_sort_directories_first() {
        let files = parse_files(
            "regular file|24\n/sdcard/a | b.jpg\0directory|4096\n/sdcard/z\0",
            "/sdcard/",
        )
        .unwrap();
        assert!(files[0].directory);
        assert_eq!(files[1].name, "a | b.jpg");
        assert_eq!(files[1].size, 24);
        assert!(parse_files("not a record", "/sdcard/").is_err());
    }

    #[test]
    fn filenames_cannot_forge_other_actionable_rows() {
        assert!(parse_files(
            "regular file|24\n/sdcard/photo\nregular file|0|0|/sdcard/important\0",
            "/sdcard"
        )
        .is_err());
        assert!(parse_files("regular file|24\n/system/important\0", "/sdcard").is_err());
        assert!(parse_files("regular file|24\n/sdcard/sub/important\0", "/sdcard").is_err());
    }
}
