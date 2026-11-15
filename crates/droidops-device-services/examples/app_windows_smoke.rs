//! Explicit, non-CI hardware check. Pass a selected serial and installed package IDs.
use droidops_core::{Device, Transport};
use droidops_device_services::{
    app_windows::{AppWindowManager, AppWindowSettings, AppWindowStatus},
    perform, Action, ProcessRunner,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let serial = args
        .next()
        .ok_or("Usage: app_windows_smoke SERIAL PACKAGE [PACKAGE...]")?;
    let values: Vec<_> = args.collect();
    let with_mirror = values.iter().any(|arg| arg == "--with-mirror");
    let packages: Vec<_> = values
        .into_iter()
        .filter(|arg| arg != "--with-mirror")
        .collect();
    if packages.is_empty() {
        return Err("Specify installed packages explicitly".into());
    }
    let device = Device {
        serial,
        model: "Selected phone".into(),
        state: "device".into(),
        transport: Transport::Adb,
    };
    let mut manager = AppWindowManager::new(Arc::new(ProcessRunner));
    let mirror_cancel = Arc::new(AtomicBool::new(false));
    let mirror = with_mirror.then(|| {
        let token = mirror_cancel.clone();
        let target = device.clone();
        tokio::spawn(async move {
            perform(&ProcessRunner, &target, &Action::Mirror, false, &token).await
        })
    });
    let result = async {
        for package in packages {
            manager.launch(
                device.clone(),
                package.clone(),
                package,
                AppWindowSettings::default(),
                false,
            )?;
        }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(40);
        loop {
            let sessions = manager.snapshot();
            if sessions
                .iter()
                .all(|s| s.status == AppWindowStatus::Running)
            {
                println!(
                    "All {} sessions verified running on virtual displays",
                    sessions.len()
                );
                break;
            }
            if sessions.iter().any(|s| {
                matches!(
                    s.status,
                    AppWindowStatus::Failed(_) | AppWindowStatus::Closed
                )
            }) || tokio::time::Instant::now() > deadline
            {
                for s in sessions {
                    println!("{}: {:?}", s.package, s.status);
                }
                return Err("One or more apps did not reach Running".into());
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        // Leave time for independent taskbar/focus/resize checks before stopping one.
        tokio::time::sleep(Duration::from_secs(20)).await;
        let sessions = manager.snapshot();
        manager.stop(sessions[0].id);
        tokio::time::sleep(Duration::from_secs(2)).await;
        let sessions = manager.snapshot();
        if sessions[0].status != AppWindowStatus::Closed
            || !sessions[1..]
                .iter()
                .all(|s| s.status == AppWindowStatus::Running)
        {
            return Err("Independent stop failed".into());
        }
        println!("Independent stop passed; remaining sessions still Running");
        if mirror.as_ref().is_some_and(|task| task.is_finished()) {
            return Err("Full-screen mirror exited during the app-window test".into());
        }
        if with_mirror {
            println!("Full-screen mirror remained active alongside app windows");
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;
    manager.shutdown().await;
    mirror_cancel.store(true, Ordering::Relaxed);
    if let Some(task) = mirror {
        let _ = task.await;
    }
    println!("Owned sessions stopped and reaped");
    result
}
