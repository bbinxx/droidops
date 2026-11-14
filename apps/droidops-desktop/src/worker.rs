use droidops_core::{Device, DeviceError, DeviceInfo, FileEntry};
use droidops_device_services::{self as services, Action, Discovery, ProcessRunner};
use eframe::egui;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tokio::sync::mpsc;

pub enum Choose {
    Install,
    Sideload,
    Upload(String),
    Download(String),
}
pub enum Task {
    Info,
    Packages(String),
    Files(String),
    Media,
    Preview(String),
    Choose(Choose),
    Action(Action, bool),
}
pub struct Job {
    pub generation: u64,
    pub device: Device,
    pub task: Task,
    cancel: Arc<AtomicBool>,
}
impl Job {
    pub fn new(generation: u64, device: Device, task: Task) -> Self {
        Self {
            generation,
            device,
            task,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}
pub enum Payload {
    Info(DeviceInfo),
    Packages(Vec<String>),
    Files(String, Vec<FileEntry>),
    Media(Vec<String>),
    Preview(egui::ColorImage),
    Chosen(Option<Action>),
    Output(String),
}
pub enum Event {
    Discovery(Discovery),
    Complete(u64, Result<Payload, DeviceError>),
    Fatal(String),
    MirrorStarted(u64),
    MirrorComplete(u64, Result<String, DeviceError>),
}

/// Runtime ownership and message passing stay outside frame rendering.
pub struct Worker {
    jobs: Option<mpsc::Sender<Job>>,
    pub events: mpsc::Receiver<Event>,
    refresh: mpsc::Sender<()>,
    cancel: Mutex<Arc<AtomicBool>>,
    shutdown: Arc<AtomicBool>,
    mirror_cancel: Arc<Mutex<Arc<AtomicBool>>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn start(ctx: egui::Context) -> Self {
        let (jobs, mut job_rx) = mpsc::channel::<Job>(1);
        let (events, event_rx) = mpsc::channel(16);
        let (refresh, mut refresh_rx) = mpsc::channel(1);
        let cancel = Mutex::new(Arc::new(AtomicBool::new(false)));
        let shutdown = Arc::new(AtomicBool::new(false));
        let mirror_cancel = Arc::new(Mutex::new(Arc::new(AtomicBool::new(false))));
        let mirror_stop = mirror_cancel.clone();
        let stop = shutdown.clone();
        let thread = thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(e) => {
                    let _ = events.blocking_send(Event::Fatal(format!(
                        "Could not start background runtime: {e}"
                    )));
                    ctx.request_repaint();
                    return;
                }
            };
            runtime.block_on(async {
                let poll_events = events.clone(); let poll_ctx = ctx.clone(); let poll_stop = stop.clone();
                let poller = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(5));
                    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    loop {
                        tokio::select! { _ = interval.tick() => {}, value = refresh_rx.recv() => { if value.is_none() { break; } } }
                        if poll_stop.load(Ordering::Relaxed) { break; }
                        let discovery = services::discover(&ProcessRunner, &poll_stop).await;
                        if poll_events.send(Event::Discovery(discovery)).await.is_err() { break; }
                        poll_ctx.request_repaint();
                    }
                });
                let decode_limit=Arc::new(tokio::sync::Semaphore::new(1));
                let mut mirror_task: Option<tokio::task::JoinHandle<()>> = None;
                while let Some(job) = job_rx.recv().await {
                    if stop.load(Ordering::Relaxed) { break; }
                    if job.cancel.load(Ordering::Relaxed) {
                        if events.send(Event::Complete(job.generation,Err(DeviceError::Cancelled))).await.is_err(){break;}
                        ctx.request_repaint();continue;
                    }
                    if matches!(job.task,Task::Action(Action::Mirror,_)) {
                        if mirror_task.as_ref().is_some_and(|task|!task.is_finished()) {
                            if events.send(Event::Complete(job.generation,Err(DeviceError::Invalid("Previous mirror is still stopping. Try again in a moment.".into())))).await.is_err(){break;}
                            ctx.request_repaint();continue;
                        }
                        let token=job.cancel.clone();
                        if let Ok(mut mirror)=mirror_stop.lock(){*mirror=token.clone();}
                        let tx=events.clone(); let repaint=ctx.clone();
                        if events.send(Event::MirrorStarted(job.generation)).await.is_err(){break;}
                        ctx.request_repaint();
                        mirror_task=Some(tokio::spawn(async move {
                            let result=services::perform(&ProcessRunner,&job.device,&Action::Mirror,false,&token).await;
                            let _=tx.send(Event::MirrorComplete(job.generation,result)).await; repaint.request_repaint();
                        }));
                        continue;
                    }
                    let result = execute(&job, &job.cancel, &decode_limit).await;
                    if events.send(Event::Complete(job.generation, result)).await.is_err() { break; }
                    ctx.request_repaint();
                }
                if let Ok(token)=mirror_stop.lock(){token.store(true,Ordering::Relaxed);}
                if let Some(task)=mirror_task {let _=task.await;}
                poller.abort(); let _ = poller.await;
            });
            runtime.shutdown_timeout(Duration::from_millis(200));
        });
        Self {
            jobs: Some(jobs),
            events: event_rx,
            refresh,
            cancel,
            shutdown,
            mirror_cancel,
            thread: Some(thread),
        }
    }

    #[cfg(test)]
    pub fn offline() -> Self {
        let (jobs, _) = mpsc::channel(1);
        let (_, events) = mpsc::channel(1);
        let (refresh, _) = mpsc::channel(1);
        Self {
            jobs: Some(jobs),
            events,
            refresh,
            cancel: Mutex::new(Arc::new(AtomicBool::new(false))),
            shutdown: Arc::new(AtomicBool::new(false)),
            mirror_cancel: Arc::new(Mutex::new(Arc::new(AtomicBool::new(false)))),
            thread: None,
        }
    }

    pub fn submit(&self, job: Job) -> Result<(), String> {
        let token = job.cancel.clone();
        let mut current = self
            .cancel
            .lock()
            .map_err(|_| "Cancellation state unavailable")?;
        self.jobs
            .as_ref()
            .ok_or("Background worker stopped")?
            .try_send(job)
            .map_err(|_| "An operation is already queued or the worker stopped")?;
        *current = token;
        Ok(())
    }
    pub fn cancel(&self) {
        if let Ok(token) = self.cancel.lock() {
            token.store(true, Ordering::Relaxed);
        }
    }
    pub fn stop_mirror(&self) {
        if let Ok(token) = self.mirror_cancel.lock() {
            token.store(true, Ordering::Relaxed);
        }
    }
    pub fn refresh(&self) {
        let _ = self.refresh.try_send(());
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.stop_mirror();
        self.cancel();
        self.jobs.take();
        // In-flight processes observe cancellation within 20 ms. File chooser futures
        // are selected against cancellation too; do not join a blocked native dialog.
        self.events.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

async fn execute(
    job: &Job,
    cancel: &AtomicBool,
    decode_limit: &Arc<tokio::sync::Semaphore>,
) -> Result<Payload, DeviceError> {
    let device = &job.device;
    match &job.task {
        Task::Info => Ok(Payload::Info(
            services::device_info(&ProcessRunner, device, cancel).await?,
        )),
        Task::Packages(filter) => Ok(Payload::Packages(
            services::packages(&ProcessRunner, device, filter, cancel).await?,
        )),
        Task::Files(path) => Ok(Payload::Files(
            path.clone(),
            services::files(&ProcessRunner, device, path, cancel).await?,
        )),
        Task::Media => Ok(Payload::Media(
            services::media(&ProcessRunner, device, cancel).await?,
        )),
        Task::Action(action, confirmed) => Ok(Payload::Output(
            services::perform(&ProcessRunner, device, action, *confirmed, cancel).await?,
        )),
        Task::Choose(kind) => {
            let choice = choose(kind);
            tokio::pin!(choice);
            loop {
                tokio::select! {
                    result = &mut choice => return Ok(Payload::Chosen(result)),
                    _ = tokio::time::sleep(Duration::from_millis(20)) => {
                        if cancel.load(Ordering::Relaxed) { return Err(DeviceError::Cancelled); }
                    }
                }
            }
        }
        Task::Preview(remote) => {
            let permit = tokio::select! {
                permit=decode_limit.clone().acquire_owned()=>permit.map_err(|_|DeviceError::Failed("Image worker stopped".into()))?,
                _=cancelled(cancel)=>return Err(DeviceError::Cancelled),
            };
            let directory = tempfile::tempdir()?;
            let local = directory.path().join("preview");
            services::perform(
                &ProcessRunner,
                device,
                &Action::Pull {
                    remote: remote.clone(),
                    local: local.clone(),
                },
                false,
                cancel,
            )
            .await?;
            if std::fs::metadata(&local)?.len() > 64 * 1024 * 1024 {
                return Err(DeviceError::Invalid(
                    "Preview is limited to images smaller than 64 MiB".into(),
                ));
            }
            let mut decode =
                tokio::task::spawn_blocking(move || -> Result<egui::ColorImage, DeviceError> {
                    let _permit = permit;
                    let _directory = directory;
                    let mut reader = image::ImageReader::open(&local)?
                        .with_guessed_format()
                        .map_err(DeviceError::Io)?;
                    let mut limits = image::Limits::default();
                    limits.max_image_width = Some(16384);
                    limits.max_image_height = Some(16384);
                    limits.max_alloc = Some(256 * 1024 * 1024);
                    reader.limits(limits);
                    let decoded = reader
                        .decode()
                        .map_err(|e| DeviceError::Invalid(format!("Cannot decode image: {e}")))?
                        .thumbnail(1280, 900)
                        .into_rgba8();
                    let size = [decoded.width() as usize, decoded.height() as usize];
                    Ok(egui::ColorImage::from_rgba_unmultiplied(
                        size,
                        decoded.as_raw(),
                    ))
                });
            loop {
                tokio::select! {
                    result=&mut decode => return Ok(Payload::Preview(result.map_err(|e|DeviceError::Failed(format!("Image worker failed: {e}")))??)),
                    _=tokio::time::sleep(Duration::from_millis(20)) => if cancel.load(Ordering::Relaxed) {decode.abort(); return Err(DeviceError::Cancelled);}
                }
            }
        }
    }
}

async fn cancelled(cancel: &AtomicBool) {
    while !cancel.load(Ordering::Relaxed) {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn choose(kind: &Choose) -> Option<Action> {
    let dialog = rfd::AsyncFileDialog::new();
    match kind {
        Choose::Install => dialog
            .set_title("Install APK")
            .add_filter("Android package", &["apk"])
            .pick_file()
            .await
            .map(|p| Action::Install(p.path().into())),
        Choose::Sideload => dialog
            .set_title("Select recovery ZIP")
            .add_filter("Recovery update", &["zip"])
            .pick_file()
            .await
            .map(|p| Action::Sideload(p.path().into())),
        Choose::Upload(remote) => dialog
            .set_title("Upload file")
            .pick_file()
            .await
            .and_then(|p| {
                let name = p.path().file_name()?.to_str()?;
                Some(Action::Push {
                    local: p.path().into(),
                    remote: format!("{}/{name}", remote.trim_end_matches('/')),
                })
            }),
        Choose::Download(remote) => {
            let name = remote.rsplit('/').next().unwrap_or("download");
            dialog
                .set_title("Save download")
                .set_file_name(name)
                .save_file()
                .await
                .map(|p| Action::Pull {
                    remote: remote.clone(),
                    local: p.path().into(),
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidops_core::Transport;
    #[test]
    fn queued_cancellation_is_never_reset_by_a_later_request() {
        let mut worker = Worker::offline();
        let (sender, mut receiver) = mpsc::channel(1);
        worker.jobs = Some(sender);
        let device = Device {
            serial: "fixture".into(),
            model: "Test".into(),
            state: "device".into(),
            transport: Transport::Adb,
        };
        worker
            .submit(Job::new(
                1,
                device.clone(),
                Task::Action(Action::Mirror, false),
            ))
            .unwrap();
        worker.cancel();
        let cancelled_job = receiver.try_recv().unwrap();
        assert!(cancelled_job.cancel.load(Ordering::Relaxed));
        worker.submit(Job::new(2, device, Task::Info)).unwrap();
        let next_job = receiver.try_recv().unwrap();
        assert!(!next_job.cancel.load(Ordering::Relaxed));
        assert!(cancelled_job.cancel.load(Ordering::Relaxed));
    }
}
