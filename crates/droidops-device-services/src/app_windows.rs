//! Owned, independent scrcpy virtual-display sessions. Applications execute on Android.
use super::{adb, Runner};
use droidops_core::{validate_package, CommandSpec, Device, DeviceError};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::{
    task::JoinHandle,
    time::{sleep, Instant},
};

/// Launch preferences apply only to new windows, never existing sessions.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AppWindowSettings {
    /// Resolve the selected phone's current main-display dimensions and density at launch.
    pub match_phone: bool,
    /// Android virtual display width in pixels.
    pub width: u32,
    /// Android virtual display height in pixels.
    pub height: u32,
    /// Initial desktop window width in pixels.
    pub window_width: u32,
    /// Initial desktop window height in pixels.
    pub window_height: u32,
    /// Android display density.
    pub dpi: u32,
    /// Maximum capture frame rate.
    pub fps: u32,
    /// Opt in to playback capture, subject to Android policy.
    pub audio: bool,
    /// Request dynamic display resizing when advertised by scrcpy.
    pub flex: bool,
    /// Show Android system decorations on the virtual display.
    pub decorations: bool,
    /// Move app content to the phone instead of destroying it on removal.
    pub preserve_content: bool,
    #[serde(skip)]
    /// Force-stop before launch; requires fresh operation-specific consent.
    pub force_stop: bool,
}
impl Default for AppWindowSettings {
    fn default() -> Self {
        Self {
            match_phone: true,
            width: 900,
            height: 1600,
            window_width: 450,
            window_height: 800,
            dpi: 240,
            fps: 60,
            audio: false,
            flex: true,
            decorations: false,
            preserve_content: false,
            force_stop: false,
        }
    }
}
/// Session state never equates process creation with successful application launch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppWindowStatus {
    Starting,
    Running,
    Stopping,
    Closed,
    Failed(String),
}
impl AppWindowStatus {
    /// Whether the manager still owns a live task for this session.
    pub fn active(&self) -> bool {
        matches!(self, Self::Starting | Self::Running | Self::Stopping)
    }
}
/// Immutable identity and current state for one independent app window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppWindowSession {
    pub id: u64,
    pub device: Device,
    pub package: String,
    pub title: String,
    pub status: AppWindowStatus,
}
struct OwnedSession {
    view: Arc<Mutex<AppWindowSession>>,
    cancel: Arc<AtomicBool>,
    task: JoinHandle<()>,
}
/// A bounded manager owned by the background runtime, never by frame rendering.
/// Drop cancels outstanding work; call shutdown before dropping the runtime to reap it.
pub struct AppWindowManager<R: Runner + 'static> {
    runner: Arc<R>,
    next: u64,
    sessions: BTreeMap<u64, OwnedSession>,
}
impl<R: Runner + 'static> AppWindowManager<R> {
    /// Construct with a production or fake process runner.
    pub fn new(runner: Arc<R>) -> Self {
        Self {
            runner,
            next: 0,
            sessions: BTreeMap::new(),
        }
    }
    /// Launch against an explicitly selected device. Force-stop consent is per launch.
    pub fn launch(
        &mut self,
        device: Device,
        package: String,
        title: String,
        settings: AppWindowSettings,
        confirmed: bool,
    ) -> Result<u64, DeviceError> {
        validate(&device, &package, &title, &settings, confirmed)?;
        if self.snapshot().iter().filter(|s| s.status.active()).count() >= 8 {
            return Err(DeviceError::Invalid(
                "At most eight app windows may be active".into(),
            ));
        }
        if self
            .snapshot()
            .iter()
            .any(|s| s.status.active() && s.device.key() == device.key() && s.package == package)
        {
            return Err(DeviceError::Invalid(
                "This app already has a window on this device".into(),
            ));
        }
        self.sessions.retain(|_, s| !s.task.is_finished());
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| DeviceError::Failed("Session IDs exhausted".into()))?;
        let id = self.next;
        let view = Arc::new(Mutex::new(AppWindowSession {
            id,
            device: device.clone(),
            package: package.clone(),
            title: title.clone(),
            status: AppWindowStatus::Starting,
        }));
        let cancel = Arc::new(AtomicBool::new(false));
        let runner = self.runner.clone();
        let state = view.clone();
        let token = cancel.clone();
        let runtime = tokio::runtime::Handle::try_current().map_err(|_| {
            DeviceError::Invalid("AppWindowManager requires a background Tokio runtime".into())
        })?;
        let task = runtime.spawn(async move {
            let result = run_session(
                runner.as_ref(),
                &device,
                &package,
                &title,
                &settings,
                &token,
                &state,
            )
            .await;
            if let Ok(mut s) = state.lock() {
                s.status = match result {
                    Ok(()) | Err(DeviceError::Cancelled) => AppWindowStatus::Closed,
                    Err(e) => AppWindowStatus::Failed(safe_error(&e)),
                };
            }
        });
        self.sessions
            .insert(id, OwnedSession { view, cancel, task });
        Ok(id)
    }
    /// Stop only this session; other windows and full-screen mirroring are independent.
    pub fn stop(&self, id: u64) {
        if let Some(s) = self.sessions.get(&id) {
            s.cancel.store(true, Ordering::Relaxed);
            if let Ok(mut v) = s.view.lock() {
                if v.status.active() {
                    v.status = AppWindowStatus::Stopping;
                }
            }
        }
    }
    /// Stop sessions whose exact device is no longer authorized and online.
    pub fn reconcile(&self, devices: &[Device]) {
        for s in self.snapshot() {
            if s.status.active()
                && !devices
                    .iter()
                    .any(|d| d.key() == s.device.key() && d.online())
            {
                self.stop(s.id);
            }
        }
    }
    /// Bounded state snapshots contain no captured shell output.
    pub fn snapshot(&self) -> Vec<AppWindowSession> {
        self.sessions
            .values()
            .filter_map(|s| s.view.lock().ok().map(|v| v.clone()))
            .collect()
    }
    /// Cancel and await every owned process before runtime teardown.
    pub async fn shutdown(&mut self) {
        for s in self.sessions.values() {
            s.cancel.store(true, Ordering::Relaxed);
        }
        for (_, s) in std::mem::take(&mut self.sessions) {
            let _ = s.task.await;
        }
    }
}
impl<R: Runner + 'static> Drop for AppWindowManager<R> {
    fn drop(&mut self) {
        for s in self.sessions.values() {
            s.cancel.store(true, Ordering::Relaxed);
        }
    }
}
fn safe_error(error: &DeviceError) -> String {
    match error {
        DeviceError::Spawn(_, _) => {
            "Could not start scrcpy or ADB. Check PATH and tool installation.".into()
        }
        DeviceError::Timeout => {
            "App launch timed out; check virtual-display compatibility and device authorization."
                .into()
        }
        DeviceError::Cancelled => "Stopped".into(),
        DeviceError::Invalid(message) => message.clone(),
        DeviceError::Failed(message) if message.contains("disabled when using") => "Conflicting scrcpy display options. Disable dynamic resizing and retry.".into(),
        DeviceError::Failed(message) if message.contains("disconnected") => "Device disconnected. Reconnect and select it before launching again.".into(),
        DeviceError::Failed(message) if message.contains("encoder") || message.contains("codec") => "Video encoding failed. Reduce virtual-display size or FPS and check device codec support.".into(),
        _ => "App window failed. Check device connection, app compatibility, and scrcpy support."
            .into(),
    }
}
fn validate(
    device: &Device,
    package: &str,
    title: &str,
    s: &AppWindowSettings,
    confirmed: bool,
) -> Result<(), DeviceError> {
    if !device.online() || device.serial.is_empty() || device.serial.chars().any(char::is_control) {
        return Err(DeviceError::Invalid(
            "Select an authorized Android device".into(),
        ));
    }
    validate_package(package)?;
    if title.is_empty()
        || title.len() > 200
        || title.chars().any(char::is_control)
        || !(320..=4096).contains(&s.width)
        || !(320..=4096).contains(&s.height)
        || !(200..=4096).contains(&s.window_width)
        || !(200..=4096).contains(&s.window_height)
        || !(72..=640).contains(&s.dpi)
        || !(1..=120).contains(&s.fps)
    {
        return Err(DeviceError::Invalid(
            "Invalid app title or display settings".into(),
        ));
    }
    if s.force_stop && !confirmed {
        return Err(DeviceError::Invalid(
            "Force-stop requires confirmation for this launch".into(),
        ));
    }
    Ok(())
}
/// Structured scrcpy arguments. Unsupported optional flags are omitted after capability probing.
pub fn command(
    device: &Device,
    package: &str,
    title: &str,
    s: &AppWindowSettings,
    help: &str,
    confirmed: bool,
) -> Result<CommandSpec, DeviceError> {
    validate(device, package, title, s, confirmed)?;
    for flag in ["--new-display", "--start-app"] {
        if !help.contains(flag) {
            return Err(DeviceError::Invalid(
                "scrcpy with virtual-display and start-app support is required".into(),
            ));
        }
    }
    let mut args = vec![
        "-s".into(),
        device.serial.clone(),
        format!("--new-display={}x{}/{}", s.width, s.height, s.dpi),
        format!(
            "--start-app={}{}",
            if s.force_stop { "+" } else { "" },
            package
        ),
        format!("--window-title={title}"),
        format!("--max-fps={}", s.fps),
        "--no-terminal-title".into(),
    ];
    if !s.audio {
        args.push("--no-audio".into());
    } else {
        args.push("--audio-source=playback".into());
    }
    // Flex defaults to unscaled rendering. Initial phone-sized frames must be
    // fitted while the virtual display catches up to a smaller host window.
    let supports_fit = help.contains("--render-fit");
    if supports_fit {
        args.push("--render-fit=letterbox".into());
    }
    if s.flex && !s.match_phone && help.contains("--flex-display") && supports_fit {
        args.push("--flex-display".into());
    } else {
        if help.contains("--no-window-aspect-ratio-lock") {
            args.push("--no-window-aspect-ratio-lock".into());
        }
        args.push(format!("--window-width={}", s.window_width));
        args.push(format!("--window-height={}", s.window_height));
    }
    if !s.decorations && help.contains("--no-vd-system-decorations") {
        args.push("--no-vd-system-decorations".into());
    }
    if s.preserve_content {
        if !help.contains("--no-vd-destroy-content") {
            return Err(DeviceError::Invalid(
                "This scrcpy cannot preserve virtual-display content".into(),
            ));
        }
        args.push("--no-vd-destroy-content".into());
    }
    let mut spec = CommandSpec::new("scrcpy", args);
    // SDL3 and SDL2 use different hints. Scope identity to this child only.
    let app_id = format!("io.github.bbinxx.droidops.app.{package}");
    spec.env = vec![
        ("SDL_APP_NAME".into(), title.into()),
        ("SDL_APP_ID".into(), app_id.clone()),
        ("SDL_VIDEO_WAYLAND_WMCLASS".into(), app_id.clone()),
        ("SDL_VIDEO_X11_WMCLASS".into(), app_id),
    ];
    spec.timeout = Duration::from_secs(24 * 60 * 60);
    Ok(spec)
}
/// Parse scrcpy's installed application labels; unknown formats safely retain package IDs.
pub fn app_names(output: &str) -> BTreeMap<String, String> {
    output
        .lines()
        .filter_map(|line| {
            let body = line
                .trim()
                .strip_prefix('*')
                .or_else(|| line.trim().strip_prefix('-'))?
                .trim();
            let (name, package) = body.rsplit_once(char::is_whitespace)?;
            validate_package(package).ok()?;
            Some((package.into(), name.trim().into()))
        })
        .collect()
}
/// Optional labels from scrcpy; callers keep ADB inventory if this tool is absent.
pub async fn names(
    runner: &impl Runner,
    device: &Device,
    cancel: &AtomicBool,
) -> BTreeMap<String, String> {
    let mut spec = CommandSpec::new(
        "scrcpy",
        vec!["-s".into(), device.serial.clone(), "--list-apps".into()],
    );
    spec.timeout = Duration::from_secs(30);
    runner
        .run(&spec, cancel)
        .await
        .map(|s| app_names(&s))
        .unwrap_or_default()
}
fn display_id(log: &str, package: &str) -> Option<u32> {
    log.lines().find_map(|line| {
        let (_, tail) = line.split_once(&format!("[{package}] on display "))?;
        tail.trim_end_matches('.')
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|id| *id > 0)
    })
}
fn resumed_on_display(output: &str, package: &str, display: u32) -> bool {
    let mut current = None;
    for line in output.lines() {
        let line = line.trim();
        if let Some(tail) = line.strip_prefix("Display #") {
            current = tail
                .split_whitespace()
                .next()
                .and_then(|s| s.trim_end_matches(':').parse::<u32>().ok());
        }
        if current == Some(display)
            && (line.contains("mResumedActivity:") || line.contains("topResumedActivity="))
            && line
                .split_whitespace()
                .any(|part| part.starts_with(&format!("{package}/")))
        {
            return true;
        }
    }
    false
}
fn display_value(output: &str, label: &str) -> Option<String> {
    for kind in ["Override", "Physical"] {
        let prefix = format!("{kind} {label}: ");
        if let Some(value) = output
            .lines()
            .find_map(|line| line.trim().strip_prefix(&prefix))
        {
            return Some(value.trim().to_owned());
        }
    }
    None
}
async fn resolve_settings(
    runner: &impl Runner,
    device: &Device,
    settings: &AppWindowSettings,
    cancel: &AtomicBool,
) -> Result<AppWindowSettings, DeviceError> {
    let mut resolved = settings.clone();
    if !settings.match_phone {
        return Ok(resolved);
    }
    let mut size = adb(device, vec!["shell".into(), "wm".into(), "size".into()]);
    size.timeout = Duration::from_secs(5);
    let mut density = adb(device, vec!["shell".into(), "wm".into(), "density".into()]);
    density.timeout = Duration::from_secs(5);
    let size = runner.run(&size, cancel).await?;
    let density = runner.run(&density, cancel).await?;
    let invalid = || {
        DeviceError::Invalid("Could not read phone resolution and density. Disable Match phone resolution to choose custom values.".into())
    };
    let dimensions = display_value(&size, "size").ok_or_else(invalid)?;
    let (width, height) = dimensions.split_once('x').ok_or_else(invalid)?;
    resolved.width = width.parse().map_err(|_| invalid())?;
    resolved.height = height.parse().map_err(|_| invalid())?;
    resolved.dpi = display_value(&density, "density")
        .ok_or_else(invalid)?
        .parse()
        .map_err(|_| invalid())?;
    Ok(resolved)
}
async fn run_session(
    runner: &impl Runner,
    device: &Device,
    package: &str,
    title: &str,
    settings: &AppWindowSettings,
    cancel: &AtomicBool,
    state: &Mutex<AppWindowSession>,
) -> Result<(), DeviceError> {
    let help = runner
        .run(&CommandSpec::new("scrcpy", vec!["--help".into()]), cancel)
        .await?;
    let settings = resolve_settings(runner, device, settings, cancel).await?;
    let spec = command(device, package, title, &settings, &help, true)?;
    let logs = Mutex::new(Vec::<u8>::new());
    let observer = |data: &[u8]| {
        if let Ok(mut bytes) = logs.lock() {
            let remaining = 65536usize.saturating_sub(bytes.len());
            bytes.extend_from_slice(&data[..remaining.min(data.len())]);
        }
    };
    let process = runner.run_observed(&spec, cancel, &observer);
    tokio::pin!(process);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut running = false;
    loop {
        tokio::select! {
            result=&mut process=>return match result {Ok(_) if running=>Ok(()),Ok(_)=>Err(DeviceError::Invalid("scrcpy closed before the app launch was verified".into())),Err(e)=>Err(e)},
            _=sleep(Duration::from_millis(300))=>{
                if cancel.load(Ordering::Relaxed){continue;}
                if !running {
                    let text=logs.lock().map(|s|String::from_utf8_lossy(&s).into_owned()).unwrap_or_default();
                if text.contains("Cannot create launch intent") || text.contains("No app found for package") || text.contains("No known display id to start app") {
                    cancel.store(true,Ordering::Relaxed);
                    let _=process.await;
                    return Err(DeviceError::Invalid("Android could not launch this app on its virtual display. It may have no launcher activity or disallow secondary displays.".into()));
                }
                let id=display_id(&text,package);
                    if let Some(id)=id {
                        let mut probe=adb(device,vec!["shell".into(),"dumpsys".into(),"activity".into(),"activities".into()]);probe.timeout=Duration::from_secs(3);
                        // Poll process and verification together: dropping this future never leaks a child.
                        let check=runner.run(&probe,cancel);tokio::pin!(check);
                        let result=tokio::select!{result=&mut process=>return result.and_then(|_|Err(DeviceError::Invalid("App window closed during verification".into()))),result=&mut check=>result};
                        if result.as_ref().is_ok_and(|out|resumed_on_display(out,package,id)) {running=true;if !cancel.load(Ordering::Relaxed) {if let Ok(mut s)=state.lock(){s.status=AppWindowStatus::Running;}}}
                    }
                    if !running&&Instant::now()>=deadline{cancel.store(true,Ordering::Relaxed);let _=process.await;return Err(DeviceError::Timeout);}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidops_core::Transport;
    const HELP:&str="--new-display --start-app --flex-display --render-fit --no-vd-system-decorations --no-vd-destroy-content";
    fn device() -> Device {
        Device {
            serial: "fixture serial".into(),
            model: "Phone".into(),
            state: "device".into(),
            transport: Transport::Adb,
        }
    }
    #[test]
    fn arguments_are_structured_and_optional_capabilities_are_respected() {
        let s = AppWindowSettings {
            match_phone: false,
            ..Default::default()
        };
        let spec = command(
            &device(),
            "org.example.app",
            "App ; $(touch unsafe)",
            &s,
            HELP,
            false,
        )
        .unwrap();
        assert!(spec.args.contains(&"--new-display=900x1600/240".into()));
        assert!(spec.args.contains(&"--start-app=org.example.app".into()));
        assert!(spec
            .args
            .contains(&"--window-title=App ; $(touch unsafe)".into()));
        assert!(spec.args.contains(&"--flex-display".into()));
        assert!(!spec
            .args
            .iter()
            .any(|arg| arg.starts_with("--window-width=") || arg.starts_with("--window-height=")));
        assert!(!command(
            &device(),
            "org.example.app",
            "Name",
            &s,
            "--new-display --start-app",
            false
        )
        .unwrap()
        .args
        .contains(&"--flex-display".into()));
        assert!(command(
            &device(),
            "org.example.app",
            "Name",
            &s,
            "--start-app",
            false
        )
        .is_err());
        assert!(command(&device(), "--bad;app", "Name", &s, HELP, false).is_err());
        let mut force = s;
        force.force_stop = true;
        assert!(command(&device(), "org.example.app", "Name", &force, HELP, false).is_err());
        assert!(
            command(&device(), "org.example.app", "Name", &force, HELP, true)
                .unwrap()
                .args
                .contains(&"--start-app=+org.example.app".into())
        );
        force.width = 0;
        assert!(command(&device(), "org.example.app", "Name", &force, HELP, true).is_err());
    }
    #[test]
    fn launch_verification_requires_exact_package_and_non_primary_display() {
        assert_eq!(
            display_id(
                "[server] INFO: Starting app \"Name\" [org.example.app] on display 3...",
                "org.example.app"
            ),
            Some(3)
        );
        assert_eq!(
            display_id(
                "Starting app [org.example.app] on display 0...",
                "org.example.app"
            ),
            None
        );
        let out="Display #0 (activities):\n mResumedActivity: ActivityRecord{x u0 org.example.app/.Main t1}\nDisplay #3 (activities):\n mResumedActivity: ActivityRecord{x u0 org.example.other/.Main t2}";
        assert!(!resumed_on_display(out, "org.example.app", 3));
        assert!(resumed_on_display(out, "org.example.other", 3));
        assert!(!resumed_on_display(out, "org.example", 3));
    }
    #[test]
    fn inventory_labels_preserve_names_and_ignore_noise() {
        let names=app_names("INFO: Apps\n * Chrome Browser     com.android.chrome\n - Music Player     org.example.music\n garbage");
        assert_eq!(names.get("com.android.chrome").unwrap(), "Chrome Browser");
        assert_eq!(names.len(), 2);
    }
    struct Fake {
        active: std::sync::atomic::AtomicUsize,
        failed: bool,
    }
    impl Runner for Fake {
        async fn run(&self, spec: &CommandSpec, _: &AtomicBool) -> Result<String, DeviceError> {
            if spec.program == "adb" && spec.args.ends_with(&["wm".into(), "size".into()]) {
                return Ok("Physical size: 1080x2400".into());
            }
            if spec.program == "adb" && spec.args.ends_with(&["wm".into(), "density".into()]) {
                return Ok("Physical density: 440".into());
            }
            if spec.args == ["--help"] {
                return Ok(HELP.into());
            }
            Ok("Display #7 (activities):\n mResumedActivity: ActivityRecord{x u0 org.example.one/.Main t1}\n Display #8 (activities):\n mResumedActivity: ActivityRecord{x u0 org.example.two/.Main t2}".into())
        }
        async fn run_observed(
            &self,
            spec: &CommandSpec,
            cancel: &AtomicBool,
            observer: &(dyn Fn(&[u8]) + Send + Sync),
        ) -> Result<String, DeviceError> {
            self.active.fetch_add(1, Ordering::SeqCst);
            let package = spec
                .args
                .iter()
                .find_map(|a| a.strip_prefix("--start-app="))
                .unwrap();
            let display = if package == "org.example.two" { 8 } else { 7 };
            observer(
                format!("Starting app \"Name\" [{package}] on display {display}...\n").as_bytes(),
            );
            if self.failed {
                self.active.fetch_sub(1, Ordering::SeqCst);
                return Err(DeviceError::Failed("Cannot create launch intent".into()));
            }
            while !cancel.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(5)).await;
            }
            self.active.fetch_sub(1, Ordering::SeqCst);
            Err(DeviceError::Cancelled)
        }
    }
    async fn wait_for(
        manager: &AppWindowManager<impl Runner + 'static>,
        test: impl Fn(&[AppWindowSession]) -> bool,
    ) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if test(&manager.snapshot()) {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn multiple_sessions_stop_independently_disconnect_and_shutdown_reap() {
        let runner = Arc::new(Fake {
            active: 0.into(),
            failed: false,
        });
        let mut manager = AppWindowManager::new(runner.clone());
        let first = manager
            .launch(
                device(),
                "org.example.one".into(),
                "One".into(),
                AppWindowSettings::default(),
                false,
            )
            .unwrap();
        let second = manager
            .launch(
                device(),
                "org.example.two".into(),
                "Two".into(),
                AppWindowSettings::default(),
                false,
            )
            .unwrap();
        assert_ne!(first, second);
        assert!(manager
            .launch(
                device(),
                "org.example.one".into(),
                "Duplicate".into(),
                AppWindowSettings::default(),
                false
            )
            .is_err());
        wait_for(&manager, |s| {
            s.iter().all(|s| s.status == AppWindowStatus::Running)
        })
        .await;
        manager.stop(first);
        wait_for(&manager, |s| s[0].status == AppWindowStatus::Closed).await;
        assert_eq!(manager.snapshot()[1].status, AppWindowStatus::Running);
        assert_eq!(runner.active.load(Ordering::SeqCst), 1);
        manager.reconcile(&[]);
        wait_for(&manager, |s| !s[1].status.active()).await;
        manager.shutdown().await;
        assert_eq!(runner.active.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn app_launch_failure_is_not_running_and_shutdown_cancels_startup() {
        let runner = Arc::new(Fake {
            active: 0.into(),
            failed: true,
        });
        let mut manager = AppWindowManager::new(runner.clone());
        manager
            .launch(
                device(),
                "org.example.one".into(),
                "One".into(),
                AppWindowSettings::default(),
                false,
            )
            .unwrap();
        wait_for(&manager, |s| {
            matches!(s[0].status, AppWindowStatus::Failed(_))
        })
        .await;
        manager.shutdown().await;
        assert_eq!(runner.active.load(Ordering::SeqCst), 0);
        let runner = Arc::new(Fake {
            active: 0.into(),
            failed: false,
        });
        let mut manager = AppWindowManager::new(runner.clone());
        manager
            .launch(
                device(),
                "org.example.one".into(),
                "One".into(),
                AppWindowSettings::default(),
                false,
            )
            .unwrap();
        sleep(Duration::from_millis(20)).await;
        manager.shutdown().await;
        assert_eq!(runner.active.load(Ordering::SeqCst), 0);
    }
    struct RejectedLaunch(Arc<Fake>);
    impl Runner for RejectedLaunch {
        async fn run(
            &self,
            spec: &CommandSpec,
            cancel: &AtomicBool,
        ) -> Result<String, DeviceError> {
            self.0.run(spec, cancel).await
        }
        async fn run_observed(
            &self,
            _: &CommandSpec,
            cancel: &AtomicBool,
            observer: &(dyn Fn(&[u8]) + Send + Sync),
        ) -> Result<String, DeviceError> {
            self.0.active.fetch_add(1, Ordering::SeqCst);
            observer(b"WARN: Cannot create launch intent for app org.example.one\n");
            while !cancel.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(5)).await;
            }
            self.0.active.fetch_sub(1, Ordering::SeqCst);
            Err(DeviceError::Cancelled)
        }
    }
    #[tokio::test]
    async fn living_window_with_rejected_app_fails_and_is_reaped() {
        let runner = Arc::new(Fake {
            active: 0.into(),
            failed: false,
        });
        let mut manager = AppWindowManager::new(Arc::new(RejectedLaunch(runner.clone())));
        manager
            .launch(
                device(),
                "org.example.one".into(),
                "One".into(),
                AppWindowSettings::default(),
                false,
            )
            .unwrap();
        wait_for(&manager, |s| {
            matches!(s[0].status, AppWindowStatus::Failed(_))
        })
        .await;
        assert_eq!(runner.active.load(Ordering::SeqCst), 0);
        manager.shutdown().await;
    }
    #[tokio::test]
    async fn session_limit_and_authorization_are_enforced_before_spawning() {
        let mut manager = AppWindowManager::new(Arc::new(Fake {
            active: 0.into(),
            failed: false,
        }));
        let mut offline = device();
        offline.state = "unauthorized".into();
        assert!(manager
            .launch(
                offline,
                "org.example.app".into(),
                "App".into(),
                AppWindowSettings::default(),
                false
            )
            .is_err());
        for index in 0..8 {
            manager
                .launch(
                    device(),
                    format!("org.example.app{index}"),
                    format!("App {index}"),
                    AppWindowSettings::default(),
                    false,
                )
                .unwrap();
        }
        assert!(manager
            .launch(
                device(),
                "org.example.ninth".into(),
                "Ninth".into(),
                AppWindowSettings::default(),
                false
            )
            .is_err());
        manager.shutdown().await;
    }
    struct DisplayRunner {
        size: &'static str,
        density: &'static str,
        calls: Mutex<Vec<CommandSpec>>,
    }
    impl Runner for DisplayRunner {
        async fn run(&self, spec: &CommandSpec, _: &AtomicBool) -> Result<String, DeviceError> {
            self.calls.lock().unwrap().push(spec.clone());
            Ok(if spec.args.last().is_some_and(|arg| arg == "size") {
                self.size
            } else {
                self.density
            }
            .into())
        }
    }
    #[tokio::test]
    async fn phone_resolution_is_default_and_reads_current_overrides_from_selected_device() {
        let runner = DisplayRunner {
            size: "Override size: 1080x2400\nPhysical size: 1440x3200",
            density: "Physical density: 560\nOverride density: 440",
            calls: Mutex::new(Vec::new()),
        };
        let settings = AppWindowSettings::default();
        assert!(settings.match_phone);
        let resolved = resolve_settings(&runner, &device(), &settings, &AtomicBool::new(false))
            .await
            .unwrap();
        assert_eq!(
            (resolved.width, resolved.height, resolved.dpi),
            (1080, 2400, 440)
        );
        for spec in runner.calls.lock().unwrap().iter() {
            assert_eq!(spec.program, "adb");
            assert_eq!(&spec.args[..3], ["-s", "fixture serial", "shell"]);
            assert_eq!(spec.args.len(), 5);
        }
        let spec = command(
            &device(),
            "org.example.app",
            "Example App",
            &resolved,
            HELP,
            false,
        )
        .unwrap();
        assert!(spec.args.contains(&"--new-display=1080x2400/440".into()));
        assert!(spec
            .env
            .contains(&("SDL_APP_NAME".into(), "Example App".into())));
        assert!(spec.env.contains(&(
            "SDL_APP_ID".into(),
            "io.github.bbinxx.droidops.app.org.example.app".into()
        )));
        let custom = AppWindowSettings {
            match_phone: false,
            flex: false,
            ..settings
        };
        runner.calls.lock().unwrap().clear();
        let resolved = resolve_settings(&runner, &device(), &custom, &AtomicBool::new(false))
            .await
            .unwrap();
        assert!(runner.calls.lock().unwrap().is_empty());
        assert_eq!(resolved.width, 900);
        let spec = command(
            &device(),
            "org.example.app",
            "App",
            &resolved,
            "--new-display --start-app --no-window-aspect-ratio-lock",
            false,
        )
        .unwrap();
        assert!(spec.args.contains(&"--no-window-aspect-ratio-lock".into()));
        assert!(spec.args.contains(&"--window-width=450".into()));
    }
    #[tokio::test]
    async fn unreadable_phone_resolution_does_not_silently_use_a_different_size() {
        let runner = DisplayRunner {
            size: "Permission denied",
            density: "Physical density: 440",
            calls: Mutex::new(Vec::new()),
        };
        assert!(resolve_settings(
            &runner,
            &device(),
            &AppWindowSettings::default(),
            &AtomicBool::new(false)
        )
        .await
        .is_err());
    }
    #[test]
    fn resizing_scales_full_frames_instead_of_cropping_phone_resolution() {
        let settings = AppWindowSettings {
            match_phone: false,
            ..Default::default()
        };
        let spec = command(&device(), "org.example.app", "App", &settings, HELP, false).unwrap();
        assert!(spec.args.contains(&"--flex-display".into()));
        assert!(spec.args.contains(&"--render-fit=letterbox".into()));
        let legacy = command(
            &device(),
            "org.example.app",
            "App",
            &settings,
            "--new-display --start-app --flex-display",
            false,
        )
        .unwrap();
        assert!(!legacy.args.contains(&"--flex-display".into()));
        assert!(!legacy
            .args
            .iter()
            .any(|arg| arg.starts_with("--render-fit=")));
        assert!(legacy.args.contains(&"--window-width=450".into()));
    }
    #[tokio::test]
    async fn matched_phone_keeps_native_density_and_scales_in_small_windows() {
        let runner = DisplayRunner {
            size: "Physical size: 1080x2400",
            density: "Physical density: 440",
            calls: Mutex::new(Vec::new()),
        };
        // Also covers saved preferences from earlier builds with flex enabled.
        let settings = AppWindowSettings {
            match_phone: true,
            flex: true,
            ..Default::default()
        };
        let resolved = resolve_settings(&runner, &device(), &settings, &AtomicBool::new(false))
            .await
            .unwrap();
        let spec = command(&device(), "org.example.app", "App", &resolved, HELP, false).unwrap();
        assert!(spec.args.contains(&"--new-display=1080x2400/440".into()));
        assert!(!spec.args.contains(&"--flex-display".into()));
        assert!(spec.args.contains(&"--window-width=450".into()));
    }
}
