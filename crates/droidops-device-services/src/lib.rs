//! Background-safe ADB/Fastboot services with an injectable process boundary.

use droidops_core::{
    parse_devices, parse_files, remote_quote, validate_package, validate_path, CommandSpec, Device,
    DeviceError, DeviceInfo, FileEntry, Transport,
};
use std::{
    future::Future,
    path::PathBuf,
    process::Stdio,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    process::Command,
    time::{sleep, Instant},
};

/// External process boundary; fakes implement the same contract without devices.
pub trait Runner: Send + Sync {
    fn run<'a>(
        &'a self,
        spec: &'a CommandSpec,
        cancel: &'a AtomicBool,
    ) -> impl Future<Output = Result<String, DeviceError>> + Send + 'a;
}

/// Native process runner. No host shell, bounded output, cancellation, and reaping.
pub struct ProcessRunner;

impl Runner for ProcessRunner {
    async fn run(&self, spec: &CommandSpec, cancel: &AtomicBool) -> Result<String, DeviceError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DeviceError::Cancelled);
        }
        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x08000000);
        let mut child = command
            .spawn()
            .map_err(|e| DeviceError::Spawn(spec.program.clone(), e.to_string()))?;
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| DeviceError::Failed("Missing stdout pipe".into()))?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| DeviceError::Failed("Missing stderr pipe".into()))?;
        let deadline = Instant::now() + spec.timeout;
        let mut out = Vec::new();
        let mut err = Vec::new();
        let mut out_buf = [0; 8192];
        let mut err_buf = [0; 8192];
        let mut out_eof = false;
        let mut err_eof = false;
        let mut status = None;
        let result = loop {
            if cancel.load(Ordering::Relaxed) {
                break Err(DeviceError::Cancelled);
            }
            if Instant::now() >= deadline {
                break Err(DeviceError::Timeout);
            }
            if out.len() + err.len() > 4 * 1024 * 1024 {
                break Err(DeviceError::OutputLimit);
            }
            if let Some(exit) = status {
                if out_eof && err_eof {
                    let exit: std::process::ExitStatus = exit;
                    if exit.success() {
                        break String::from_utf8(out).map_err(|_| {
                            DeviceError::Invalid("Command returned non-UTF-8 data".into())
                        });
                    }
                    let detail = if err.is_empty() { &out } else { &err };
                    let text: String = String::from_utf8_lossy(detail).chars().take(4096).collect();
                    break Err(DeviceError::Failed(if text.trim().is_empty() {
                        format!("{} exited with {exit}", spec.program)
                    } else {
                        text
                    }));
                }
            }
            tokio::select! {
                read = stdout.read(&mut out_buf), if !out_eof => match read {
                    Ok(0) => out_eof = true, Ok(n) => out.extend_from_slice(&out_buf[..n]), Err(e) => break Err(e.into()),
                },
                read = stderr.read(&mut err_buf), if !err_eof => match read {
                    Ok(0) => err_eof = true, Ok(n) => err.extend_from_slice(&err_buf[..n]), Err(e) => break Err(e.into()),
                },
                exit = child.wait(), if status.is_none() => match exit { Ok(exit) => status = Some(exit), Err(e) => break Err(e.into()) },
                _ = sleep(Duration::from_millis(20)) => {},
            }
        };
        if result.is_err() && status.is_none() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        result
    }
}

/// Discovery results preserve partial tool failures rather than hiding them.
#[derive(Debug, Default)]
pub struct Discovery {
    pub devices: Vec<Device>,
    pub warnings: Vec<String>,
}

/// Read-only discovery across both transports.
pub async fn discover(runner: &impl Runner, cancel: &AtomicBool) -> Discovery {
    let mut result = Discovery::default();
    for (program, args, transport) in [
        ("adb", vec!["devices".into(), "-l".into()], Transport::Adb),
        ("fastboot", vec!["devices".into()], Transport::Fastboot),
    ] {
        let mut spec = CommandSpec::new(program, args);
        spec.timeout = Duration::from_secs(5);
        match runner
            .run(&spec, cancel)
            .await
            .and_then(|out| parse_devices(&out, transport))
        {
            Ok(devices) => result.devices.extend(devices),
            Err(e) => result.warnings.push(e.to_string()),
        }
    }
    result
}

fn adb(device: &Device, args: Vec<String>) -> CommandSpec {
    let mut selected = vec!["-s".into(), device.serial.clone()];
    selected.extend(args);
    CommandSpec::new("adb", selected)
}

fn shell(device: &Device, args: &[&str]) -> Result<CommandSpec, DeviceError> {
    let script = args
        .iter()
        .map(|s| remote_quote(s))
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    Ok(adb(device, vec!["shell".into(), script]))
}

fn require_online(device: &Device) -> Result<(), DeviceError> {
    if !device.online() {
        return Err(DeviceError::Invalid(
            "Select an authorized, online ADB device".into(),
        ));
    }
    Ok(())
}

/// Fetch dashboard properties without placing process work in a UI frame.
pub async fn device_info(
    runner: &impl Runner,
    device: &Device,
    cancel: &AtomicBool,
) -> Result<DeviceInfo, DeviceError> {
    require_online(device)?;
    let model = runner
        .run(&shell(device, &["getprop", "ro.product.model"])?, cancel)
        .await?;
    let android = runner
        .run(
            &shell(device, &["getprop", "ro.build.version.release"])?,
            cancel,
        )
        .await?;
    let battery = runner
        .run(&shell(device, &["dumpsys", "battery"])?, cancel)
        .await?;
    let value = |key: &str| {
        battery
            .lines()
            .find_map(|line| line.trim().strip_prefix(key))
            .unwrap_or("Unknown")
            .trim()
            .to_owned()
    };
    let charging = match value("status:").as_str() {
        "2" => "Charging",
        "3" => "Discharging",
        "4" => "Not charging",
        "5" => "Full",
        _ => "Unknown",
    };
    Ok(DeviceInfo {
        model: model.trim().into(),
        android: android.trim().into(),
        battery: value("level:"),
        charging: charging.into(),
    })
}

/// Package inventory; names remain truthful package IDs until resource decoding exists.
pub async fn packages(
    runner: &impl Runner,
    device: &Device,
    filter: &str,
    cancel: &AtomicBool,
) -> Result<Vec<String>, DeviceError> {
    require_online(device)?;
    let mut args = vec!["pm", "list", "packages"];
    match filter {
        "User" => args.push("-3"),
        "System" => args.push("-s"),
        "All" => {}
        _ => return Err(DeviceError::Invalid("Invalid package filter".into())),
    }
    let out = runner.run(&shell(device, &args)?, cancel).await?;
    let mut result = Vec::new();
    for line in out.lines().filter(|l| !l.trim().is_empty()) {
        let name = line
            .trim()
            .strip_prefix("package:")
            .ok_or_else(|| DeviceError::Invalid("Malformed package list".into()))?;
        validate_package(name)?;
        result.push(name.to_owned());
    }
    result.sort();
    Ok(result)
}

/// List files with fixed-prefix stat records rather than locale-dependent ls columns.
pub async fn files(
    runner: &impl Runner,
    device: &Device,
    path: &str,
    cancel: &AtomicBool,
) -> Result<Vec<FileEntry>, DeviceError> {
    require_online(device)?;
    validate_path(path, false)?;
    let mut spec = shell(
        device,
        &[
            "find",
            path,
            "-mindepth",
            "1",
            "-maxdepth",
            "1",
            "-exec",
            "sh",
            "-c",
            "for p do stat -c '%F|%s' -- \"$p\" || exit 1; printf '%s\\0' \"$p\"; done",
            "sh",
            "{}",
            "+",
        ],
    )?;
    spec.timeout = Duration::from_secs(60);
    parse_files(&runner.run(&spec, cancel).await?, path)
}

/// Find image paths in shared DCIM/Pictures storage; paths are not interpolated input.
pub async fn media(
    runner: &impl Runner,
    device: &Device,
    cancel: &AtomicBool,
) -> Result<Vec<String>, DeviceError> {
    require_online(device)?;
    let script = "for p in /sdcard/DCIM /sdcard/Pictures; do if [ -d \"$p\" ]; then find \"$p\" -type f \\( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' -o -iname '*.webp' \\) -print0; fi; done";
    let mut spec = adb(device, vec!["shell".into(), script.into()]);
    spec.timeout = Duration::from_secs(60);
    let out = runner.run(&spec, cancel).await?;
    let mut paths = Vec::new();
    for line in out.split_terminator('\0') {
        validate_path(line, false)?;
        paths.push(line.into());
    }
    paths.sort();
    Ok(paths)
}

/// Managed actions supported by the current application. No flashing/unlocking API.
#[derive(Clone, Debug)]
pub enum Action {
    Install(PathBuf),
    Uninstall(String),
    ClearData(String),
    Sideload(PathBuf),
    Pull { remote: String, local: PathBuf },
    Push { local: PathBuf, remote: String },
    Delete(String),
    CreateDirectory(String),
    Rename { from: String, to: String },
    Shell(String),
    Reboot(String),
    Mirror,
}

impl Action {
    /// Whether submission needs operation-specific user confirmation.
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self,
            Self::Install(_)
                | Self::Push { .. }
                | Self::Uninstall(_)
                | Self::ClearData(_)
                | Self::Delete(_)
                | Self::Rename { .. }
                | Self::Sideload(_)
                | Self::Reboot(_)
                | Self::Shell(_)
        )
    }

    /// Human-readable confirmation text. Kept out of logging.
    pub fn description(&self) -> String {
        match self {
            Self::Install(path) => format!(
                "Install {} (replace an existing version if present)",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            Self::Uninstall(p) => format!("Uninstall {p}"),
            Self::ClearData(p) => format!("Permanently clear data for {p}"),
            Self::Sideload(path) => format!(
                "Apply recovery ZIP update {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            Self::Pull { .. } => "Download file".into(),
            Self::Push { remote, .. } => {
                format!("Upload to {remote}. An existing file at this destination will be replaced")
            }
            Self::Delete(p) => format!("Permanently delete {p}"),
            Self::CreateDirectory(_) => "Create folder".into(),
            Self::Rename { from, to } => format!("Rename {from} to {to}"),
            Self::Shell(_) => "Execute the entered shell command (it may modify the device)".into(),
            Self::Reboot(mode) => format!(
                "Reboot device to {}",
                if mode.is_empty() { "system" } else { mode }
            ),
            Self::Mirror => "Mirror screen with scrcpy".into(),
        }
    }
}

fn host_path(path: &std::path::Path) -> Result<String, DeviceError> {
    if !path.is_absolute() {
        return Err(DeviceError::Invalid("Select an absolute local path".into()));
    }
    let path = path
        .to_str()
        .ok_or_else(|| DeviceError::Invalid("Local path is not valid Unicode".into()))?;
    if path.contains('\0') {
        return Err(DeviceError::Invalid("Invalid local path".into()));
    }
    Ok(path.into())
}

fn mutation_guard(paths: &[&str]) -> Result<String, DeviceError> {
    let mut script=String::from("root=$(readlink -f /sdcard) || exit 1; [ -n \"$root\" ] || exit 1; safe_parent() { [ ! -L \"$1\" ] || { echo 'Symlink targets cannot be modified' >&2; return 1; }; parent=$(readlink -f -- \"$(dirname -- \"$1\")\") || return 1; case \"$parent/\" in \"$root/\"*) ;; *) echo 'Path resolves outside shared storage' >&2; return 1;; esac; }; ");
    for path in paths {
        validate_path(path, true)?;
        script.push_str(&format!(
            "safe_parent {} || exit 1; ",
            remote_quote(path.trim_end_matches('/'))?
        ));
    }
    Ok(script)
}

fn guarded_shell(
    device: &Device,
    paths: &[&str],
    args: &[&str],
) -> Result<CommandSpec, DeviceError> {
    let command = args
        .iter()
        .map(|s| remote_quote(s))
        .collect::<Result<Vec<_>, _>>()?
        .join(" ");
    Ok(adb(
        device,
        vec!["shell".into(), mutation_guard(paths)? + &command],
    ))
}

/// Construct a validated command; the UI passes confirmation only after a bound dialog.
pub fn action_command(
    device: &Device,
    action: &Action,
    confirmed: bool,
) -> Result<CommandSpec, DeviceError> {
    if device.serial.is_empty()
        || device.serial.starts_with('-')
        || device.serial.chars().any(char::is_whitespace)
    {
        return Err(DeviceError::Invalid("Invalid device selection".into()));
    }
    if action.requires_confirmation() && !confirmed {
        return Err(DeviceError::Invalid(
            "Explicit confirmation is required".into(),
        ));
    }
    if let Action::Reboot(mode) = action {
        if !["", "bootloader", "recovery"].contains(&mode.as_str()) {
            return Err(DeviceError::Invalid("Unsupported reboot mode".into()));
        }
        if device.transport == Transport::Fastboot {
            if !mode.is_empty() {
                return Err(DeviceError::Invalid(
                    "Only reboot to system is supported from Fastboot".into(),
                ));
            }
            return Ok(CommandSpec::new(
                "fastboot",
                vec!["-s".into(), device.serial.clone(), "reboot".into()],
            ));
        }
    }
    if matches!(action, Action::Sideload(_)) {
        if device.transport != Transport::Adb || device.state != "sideload" {
            return Err(DeviceError::Invalid(
                "Put the selected device in Apply update from ADB mode first".into(),
            ));
        }
    } else {
        require_online(device)?;
    }
    let mut spec = match action {
        Action::Install(path) | Action::Sideload(path) => {
            let expected = if matches!(action, Action::Install(_)) {
                "apk"
            } else {
                "zip"
            };
            if !path
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case(expected))
            {
                return Err(DeviceError::Invalid(format!("Select a .{expected} file")));
            }
            let mut args = if expected == "apk" {
                vec!["install".into(), "-r".into()]
            } else {
                vec!["sideload".into()]
            };
            args.push(host_path(path)?);
            let mut cmd = adb(device, args);
            cmd.timeout = Duration::from_secs(900);
            cmd
        }
        Action::Uninstall(package) => {
            validate_package(package)?;
            adb(device, vec!["uninstall".into(), package.clone()])
        }
        Action::ClearData(package) => {
            validate_package(package)?;
            shell(device, &["pm", "clear", package])?
        }
        Action::Pull { remote, local } => {
            validate_path(remote, false)?;
            adb(
                device,
                vec!["pull".into(), remote.clone(), host_path(local)?],
            )
        }
        Action::Push { local, remote } => {
            validate_path(remote, true)?;
            adb(
                device,
                vec!["push".into(), host_path(local)?, remote.clone()],
            )
        }
        Action::Delete(path) => {
            validate_path(path, true)?;
            guarded_shell(
                device,
                &[path],
                &["rm", "-rf", "--", path.trim_end_matches('/')],
            )?
        }
        Action::CreateDirectory(path) => {
            validate_path(path, true)?;
            guarded_shell(
                device,
                &[path],
                &["mkdir", "-p", "--", path.trim_end_matches('/')],
            )?
        }
        Action::Rename { from, to } => {
            validate_path(from, true)?;
            validate_path(to, true)?;
            {
                let source = remote_quote(from)?;
                let destination = remote_quote(to)?;
                let script=mutation_guard(&[from,to])?+&format!("if [ -e {destination} ] || [ -L {destination} ]; then echo 'Rename destination already exists' >&2; exit 1; fi; mv -T -n -- {source} {destination} || exit 1; [ ! -e {source} ] && [ -e {destination} ] || {{ echo 'Rename did not complete' >&2; exit 1; }}");
                adb(device, vec!["shell".into(), script])
            }
        }
        Action::Shell(script) => {
            if script.trim().is_empty() || script.contains('\0') {
                return Err(DeviceError::Invalid("Enter a shell command".into()));
            }
            adb(device, vec!["shell".into(), script.clone()])
        }
        Action::Reboot(mode) => {
            let mut args = vec!["reboot".into()];
            if !mode.is_empty() {
                args.push(mode.clone());
            }
            adb(device, args)
        }
        Action::Mirror => CommandSpec::new(
            "scrcpy",
            vec![
                "-s".into(),
                device.serial.clone(),
                "--stay-awake".into(),
                "--always-on-top".into(),
            ],
        ),
    };
    if matches!(action, Action::Pull { .. } | Action::Push { .. }) {
        spec.timeout = Duration::from_secs(900);
    }
    if matches!(action, Action::Mirror) {
        spec.timeout = Duration::from_secs(24 * 60 * 60);
    }
    Ok(spec)
}

/// Execute a validated operation and recognize Android tool failure markers.
pub async fn perform(
    runner: &impl Runner,
    device: &Device,
    action: &Action,
    confirmed: bool,
    cancel: &AtomicBool,
) -> Result<String, DeviceError> {
    let spec = action_command(device, action, confirmed)?;
    tracing::debug!(program = spec.program, "starting device operation");
    if let Action::Push { remote, local } = action {
        if !std::fs::metadata(local)?.is_file() {
            return Err(DeviceError::Invalid(
                "Select a regular file; recursive directory upload is not supported".into(),
            ));
        }
        let destination = remote_quote(remote)?;
        let preflight=mutation_guard(&[remote])?+&format!("if [ -d {destination} ]; then echo 'Upload destination is a directory' >&2; exit 1; fi");
        runner
            .run(&adb(device, vec!["shell".into(), preflight]), cancel)
            .await?;
    }
    let out = runner.run(&spec, cancel).await?;
    if out
        .lines()
        .any(|line| line.trim().starts_with("Failure") || line.trim() == "Failed")
    {
        return Err(DeviceError::Failed(out));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn device() -> Device {
        Device {
            serial: "fixture-device".into(),
            model: "Fixture".into(),
            state: "device".into(),
            transport: Transport::Adb,
        }
    }
    struct Fake {
        commands: Mutex<Vec<CommandSpec>>,
        response: String,
    }
    impl Runner for Fake {
        async fn run(&self, spec: &CommandSpec, _: &AtomicBool) -> Result<String, DeviceError> {
            self.commands.lock().unwrap().push(spec.clone());
            Ok(self.response.clone())
        }
    }

    #[test]
    fn destructive_actions_require_confirmation_and_valid_target() {
        let action = Action::Delete("/sdcard/a' ; id".into());
        assert!(action_command(&device(), &action, false).is_err());
        let spec = action_command(&device(), &action, true).unwrap();
        assert_eq!(&spec.args[..3], ["-s", "fixture-device", "shell"]);
        assert!(spec.args[3].contains("Path resolves outside shared storage"));
        assert!(spec.args[3].ends_with("'rm' '-rf' '--' '/sdcard/a'\\'' ; id'"));
        assert!(action_command(&device(), &Action::Delete("/sdcard/".into()), true).is_err());
        let mut offline = device();
        offline.state = "unauthorized".into();
        assert!(action_command(&offline, &action, true).is_err());
    }

    #[test]
    fn bootloader_surface_cannot_flash_or_unlock() {
        let mut boot = device();
        boot.transport = Transport::Fastboot;
        boot.state = "fastboot".into();
        let cmd = action_command(&boot, &Action::Reboot("".into()), true).unwrap();
        assert_eq!(cmd.program, "fastboot");
        assert_eq!(cmd.args, ["-s", "fixture-device", "reboot"]);
        assert!(action_command(&boot, &Action::Shell("flash boot".into()), true).is_err());
        assert!(action_command(&device(), &Action::Reboot("unlock".into()), true).is_err());
    }

    #[tokio::test]
    async fn android_failure_output_is_not_reported_as_success() {
        let fake = Fake {
            commands: Mutex::default(),
            response: "Failure [DELETE_FAILED_INTERNAL_ERROR]\n".into(),
        };
        let result = perform(
            &fake,
            &device(),
            &Action::Uninstall("com.example.app".into()),
            true,
            &AtomicBool::new(false),
        )
        .await;
        assert!(matches!(result, Err(DeviceError::Failed(_))));
        assert_eq!(
            fake.commands.lock().unwrap()[0].args,
            ["-s", "fixture-device", "uninstall", "com.example.app"]
        );
    }

    #[tokio::test]
    async fn package_inventory_parses_and_filters_without_a_device() {
        let fake = Fake {
            commands: Mutex::default(),
            response: "package:com.z.app\npackage:com.a.app\n".into(),
        };
        let result = packages(&fake, &device(), "User", &AtomicBool::new(false))
            .await
            .unwrap();
        assert_eq!(result, ["com.a.app", "com.z.app"]);
        assert_eq!(
            fake.commands.lock().unwrap()[0].args[3],
            "'pm' 'list' 'packages' '-3'"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn real_runner_times_out_and_honors_cancellation_without_devices() {
        let mut spec = CommandSpec::new("/bin/sleep", vec!["5".into()]);
        spec.timeout = Duration::from_millis(50);
        assert!(matches!(
            ProcessRunner.run(&spec, &AtomicBool::new(false)).await,
            Err(DeviceError::Timeout)
        ));
        assert!(matches!(
            ProcessRunner.run(&spec, &AtomicBool::new(true)).await,
            Err(DeviceError::Cancelled)
        ));
        let out = ProcessRunner
            .run(
                &CommandSpec::new("/bin/echo", vec!["literal ; $(id)".into()]),
                &AtomicBool::new(false),
            )
            .await
            .unwrap();
        assert_eq!(out, "literal ; $(id)\n");
    }

    #[test]
    fn uploads_and_apk_replacement_require_explicit_consent() {
        let upload = Action::Push {
            local: PathBuf::from("/tmp/fixture.txt"),
            remote: "/sdcard/fixture.txt".into(),
        };
        assert!(action_command(&device(), &upload, false).is_err());
        assert!(upload.description().contains("existing file"));
        assert!(action_command(
            &device(),
            &Action::Install(PathBuf::from("/tmp/fixture.apk")),
            false
        )
        .is_err());
    }

    #[tokio::test]
    async fn recursive_upload_is_rejected_before_any_device_command() {
        let directory = tempfile::tempdir().unwrap();
        let fake = Fake {
            commands: Mutex::default(),
            response: String::new(),
        };
        let result = perform(
            &fake,
            &device(),
            &Action::Push {
                local: directory.path().into(),
                remote: "/sdcard/upload".into(),
            },
            true,
            &AtomicBool::new(false),
        )
        .await;
        assert!(matches!(result, Err(DeviceError::Invalid(_))));
        assert!(fake.commands.lock().unwrap().is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn managed_delete_cannot_follow_parent_symlinks_outside_shared_storage() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let shared = temp.path().join("shared");
        let outside = temp.path().join("outside");
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let victim = outside.join("keep.txt");
        std::fs::write(&victim, "keep").unwrap();
        symlink(&outside, shared.join("link")).unwrap();
        let spec = action_command(
            &device(),
            &Action::Delete("/sdcard/link/keep.txt".into()),
            true,
        )
        .unwrap();
        // Execute the real emitted script only against a disposable host fixture.
        let script = spec.args[3].replace("/sdcard", shared.to_str().unwrap());
        let result = ProcessRunner
            .run(
                &CommandSpec::new("/bin/sh", vec!["-c".into(), script]),
                &AtomicBool::new(false),
            )
            .await;
        assert!(matches!(result, Err(DeviceError::Failed(_))));
        assert_eq!(std::fs::read_to_string(victim).unwrap(), "keep");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rename_rejects_existing_files_and_directories_without_a_successful_noop() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("from");
        let target = temp.path().join("to");
        std::fs::write(&source, "original").unwrap();
        std::fs::write(&target, "existing").unwrap();
        let spec = action_command(
            &device(),
            &Action::Rename {
                from: "/sdcard/from".into(),
                to: "/sdcard/to".into(),
            },
            true,
        )
        .unwrap();
        let script = spec.args[3].replace("/sdcard", temp.path().to_str().unwrap());
        let command = CommandSpec::new("/bin/sh", vec!["-c".into(), script]);
        assert!(ProcessRunner
            .run(&command, &AtomicBool::new(false))
            .await
            .is_err());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "existing");
        std::fs::remove_file(&target).unwrap();
        std::fs::create_dir(&target).unwrap();
        assert!(ProcessRunner
            .run(&command, &AtomicBool::new(false))
            .await
            .is_err());
        assert!(source.is_file());
        assert!(!target.join("from").exists());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn runner_bounds_output_and_reports_spawn_and_encoding_errors() {
        let cancel = AtomicBool::new(false);
        let overflow = CommandSpec::new(
            "/bin/sh",
            vec!["-c".into(), "head -c 5242880 /dev/zero".into()],
        );
        assert!(matches!(
            ProcessRunner.run(&overflow, &cancel).await,
            Err(DeviceError::OutputLimit)
        ));
        let missing = CommandSpec::new("/droidops-nonexistent-fixture-tool", Vec::new());
        assert!(matches!(
            ProcessRunner.run(&missing, &cancel).await,
            Err(DeviceError::Spawn(..))
        ));
        let non_utf8 = CommandSpec::new("/bin/sh", vec!["-c".into(), "printf '\\377'".into()]);
        assert!(matches!(
            ProcessRunner.run(&non_utf8, &cancel).await,
            Err(DeviceError::Invalid(_))
        ));
    }
}
