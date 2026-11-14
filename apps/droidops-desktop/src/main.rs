#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod icons;
mod screens;
mod state;
mod worker;

use droidops_core::{Device, DeviceInfo, FileEntry, Transport};
use droidops_device_services::Action;
use eframe::egui::{self, Color32, RichText, Vec2};
use icons::{Icon, IconButton};
use state::{Preferences, Screen, Session};
use worker::{Choose, Event, Job, Payload, Task, Worker};

const ACCENT: Color32 = Color32::from_rgb(0, 87, 224);

fn main() -> eframe::Result {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .try_init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DroidOps")
            .with_icon(icons::window_icon())
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([800.0, 580.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "DroidOps",
        options,
        Box::new(|cc| Ok(Box::new(DroidOps::new(cc)))),
    )
}

struct Pending {
    action: Action,
    generation: u64,
}
struct FileDialog {
    original: Option<String>,
    name: String,
}

struct DroidOps {
    worker: Worker,
    session: Session,
    devices: Vec<Device>,
    warnings: Vec<String>,
    screen: Screen,
    preferences: Preferences,
    busy: bool,
    mirroring: bool,
    operation: String,
    status: String,
    error: Option<String>,
    pending: Option<Pending>,
    info: Option<DeviceInfo>,
    apps: Vec<String>,
    app_filter: String,
    search: String,
    path: String,
    path_draft: String,
    files: Vec<FileEntry>,
    media: Vec<String>,
    album: String,
    shell: String,
    output: String,
    preview_path: String,
    preview: Option<egui::TextureHandle>,
    file_dialog: Option<FileDialog>,
    reload: bool,
}

impl DroidOps {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut preferences: Preferences = cc
            .storage
            .and_then(|s| eframe::get_value(s, "preferences"))
            .unwrap_or_default();
        if !preferences.scale.is_finite() {
            preferences.scale = 1.0;
        }
        preferences.scale = preferences.scale.clamp(0.8, 1.6);
        theme(&cc.egui_ctx, &preferences);
        Self::with_worker(Worker::start(cc.egui_ctx.clone()), preferences)
    }

    fn with_worker(worker: Worker, preferences: Preferences) -> Self {
        Self {
            worker,
            session: Session::default(),
            devices: Vec::new(),
            warnings: Vec::new(),
            screen: Screen::Dashboard,
            preferences,
            busy: false,
            mirroring: false,
            operation: String::new(),
            status: "Looking for devices…".into(),
            error: None,
            pending: None,
            info: None,
            apps: Vec::new(),
            app_filter: "User".into(),
            search: String::new(),
            path: "/sdcard/".into(),
            path_draft: "/sdcard/".into(),
            files: Vec::new(),
            media: Vec::new(),
            album: "All albums".into(),
            shell: String::new(),
            output: String::new(),
            preview_path: String::new(),
            preview: None,
            file_dialog: None,
            reload: false,
        }
    }

    fn submit(&mut self, task: Task, label: &str) {
        if self.busy {
            return;
        }
        let Some(device) = self.session.device.clone() else {
            self.error = Some("Choose a device first.".into());
            return;
        };
        match self
            .worker
            .submit(Job::new(self.session.generation, device, task))
        {
            Ok(()) => {
                self.busy = true;
                self.operation = label.into();
                self.status = label.into();
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn action(&mut self, action: Action) {
        if self.busy {
            return;
        }
        if action.requires_confirmation() {
            self.pending = Some(Pending {
                action,
                generation: self.session.generation,
            });
        } else {
            let label = action.description();
            self.submit(Task::Action(action, false), &label);
        }
    }

    fn select(&mut self, device: Option<Device>) {
        self.worker.cancel();
        self.worker.stop_mirror();
        self.mirroring = false;
        self.session.select(device);
        self.clear_device_state();
        self.reload = true;
    }

    fn clear_device_state(&mut self) {
        self.info = None;
        self.apps.clear();
        self.files.clear();
        self.media.clear();
        self.pending = None;
        self.preview = None;
        self.preview_path.clear();
        self.file_dialog = None;
        self.output.clear();
        self.shell.clear();
        self.search.clear();
        self.path = "/sdcard/".into();
        self.path_draft = self.path.clone();
        self.album = "All albums".into();
    }

    fn receive(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.worker.events.try_recv() {
            match event {
                Event::Discovery(discovery) => {
                    if self.session.reconcile(&discovery.devices) {
                        self.worker.cancel();
                        self.worker.stop_mirror();
                        self.mirroring = false;
                        self.clear_device_state();
                        self.status = "Device state changed. Select a device to continue.".into();
                    } else if self.session.device.is_none() && !self.busy {
                        self.status = if discovery.devices.is_empty() {
                            "No devices connected".into()
                        } else {
                            "Choose a device to get started".into()
                        };
                    }
                    self.devices = discovery.devices;
                    self.warnings = discovery.warnings;
                }
                Event::MirrorStarted(generation) => {
                    self.busy = false;
                    if self.session.accepts(generation) {
                        self.mirroring = true;
                        self.status = "Screen mirroring active".into();
                    }
                }
                Event::MirrorComplete(generation, result) => {
                    if self.session.accepts(generation) {
                        self.mirroring = false;
                        if let Err(error) = result {
                            if !matches!(error, droidops_core::DeviceError::Cancelled) {
                                self.error = Some(error.to_string());
                            }
                        }
                    }
                }
                Event::Fatal(message) => {
                    self.busy = false;
                    self.error = Some(message);
                }
                Event::Complete(generation, result) => {
                    self.busy = false;
                    if !self.session.accepts(generation) {
                        continue;
                    }
                    match result {
                        Ok(payload) => {
                            self.status = "Ready".into();
                            match payload {
                                Payload::Info(info) => self.info = Some(info),
                                Payload::Packages(apps) => self.apps = apps,
                                Payload::Files(path, files) => {
                                    self.path = path;
                                    self.path_draft = self.path.clone();
                                    self.files = files;
                                }
                                Payload::Media(media) => self.media = media,
                                Payload::Preview(image) => {
                                    self.preview = Some(ctx.load_texture(
                                        "photo-preview",
                                        image,
                                        egui::TextureOptions::LINEAR,
                                    ))
                                }
                                Payload::Chosen(Some(action)) => self.action(action),
                                Payload::Chosen(None) => {}
                                Payload::Output(out) => {
                                    self.status = format!("{} complete", self.operation);
                                    append_output(&mut self.output, &out);
                                    self.reload = self.screen != Screen::Shell;
                                }
                            }
                        }
                        Err(droidops_core::DeviceError::Cancelled) => {
                            self.status =
                                "Cancelled. Changes on the device may be incomplete.".into()
                        }
                        Err(error) => {
                            self.status = "Operation failed".into();
                            self.error = Some(error.to_string());
                        }
                    }
                }
            }
        }
        if self.reload && !self.busy {
            self.reload = false;
            if self.session.device.as_ref().is_some_and(Device::online) {
                match self.screen {
                    Screen::Dashboard => self.submit(Task::Info, "Reading device details"),
                    Screen::Applications => self.submit(
                        Task::Packages(self.app_filter.clone()),
                        "Loading applications",
                    ),
                    Screen::Files => {
                        self.submit(Task::Files(self.path.clone()), "Reading directory")
                    }
                    Screen::Gallery => self.submit(Task::Media, "Finding photos"),
                    _ => {}
                }
            }
        }
    }

    fn navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.available_width() < 1140.0 {
                icons::show(ui, screen_icon(self.screen), ACCENT);
                let previous = self.screen;
                egui::ComboBox::from_id_salt("navigation")
                    .width(240.0)
                    .selected_text(self.screen.title())
                    .show_ui(ui, |ui| {
                        for screen in Screen::ALL {
                            ui.selectable_value(&mut self.screen, screen, screen.title());
                        }
                    });
                if self.screen != previous {
                    self.reload = true;
                    self.search.clear();
                }
            } else {
                for screen in Screen::ALL {
                    let selected = self.screen == screen;
                    if ui
                        .add(
                            IconButton::new(screen_icon(screen), screen.title()).selected(selected),
                        )
                        .clicked()
                        && !selected
                    {
                        self.screen = screen;
                        self.reload = true;
                        self.search.clear();
                    }
                }
            }
        });
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Workspace")
                    .size(14.0)
                    .color(ui.visuals().weak_text_color()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(IconButton::new(Icon::Refresh, "Refresh devices"))
                    .clicked()
                {
                    self.worker.refresh();
                }
                let mut selection = self.session.device.as_ref().map(Device::key);
                let previous = selection.clone();
                let title = self
                    .session
                    .device
                    .as_ref()
                    .map(|d| format!("{} · {}", d.model, d.state))
                    .unwrap_or_else(|| "Select a device".into());
                egui::ComboBox::from_id_salt("device-selector")
                    .width(280.0)
                    .selected_text(title)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selection, None, "Select a device");
                        for device in &self.devices {
                            ui.selectable_value(
                                &mut selection,
                                Some(device.key()),
                                format!("{} · {} · {}", device.model, device.serial, device.state),
                            );
                        }
                    });
                if previous != selection {
                    let device = self
                        .devices
                        .iter()
                        .find(|d| Some(d.key()) == selection)
                        .cloned();
                    self.select(device);
                }
                icons::show(
                    ui,
                    Icon::Device,
                    if self.session.device.is_some() {
                        ACCENT
                    } else {
                        Color32::GRAY
                    },
                );
            });
        });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if let Some(pending) = &self.pending {
            let description = pending.action.description();
            let generation = pending.generation;
            let device = self
                .session
                .device
                .as_ref()
                .map(|d| format!("{} ({})", d.model, d.serial))
                .unwrap_or_default();
            let mut confirm = false;
            let mut dismiss = false;
            egui::Modal::new(egui::Id::unique("confirm-operation")).show(ctx,|ui| {
                ui.set_width(440.0); ui.heading("Confirm device operation"); ui.add_space(12.0);
                ui.label(description); ui.label(RichText::new(device).strong());
                ui.add_space(8.0); ui.weak("Changes may be permanent. Cancelling an operation does not undo changes already made.");
                ui.add_space(16.0); ui.horizontal(|ui| {
                    dismiss=ui.add(IconButton::new(Icon::Stop,"Cancel")).clicked();
                    confirm=ui.add_enabled(self.session.accepts(generation) && !self.busy,primary("CONFIRM AND CONTINUE")).clicked();
                });
            });
            if confirm {
                if let Some(pending) = self.pending.take() {
                    let label = pending.action.description();
                    self.submit(Task::Action(pending.action, true), &label);
                }
            } else if dismiss {
                self.pending = None;
            }
        }
        if let Some(error) = self.error.clone() {
            egui::Modal::new(egui::Id::unique("operation-error")).show(ctx, |ui| {
                ui.set_width(460.0);
                ui.heading("Operation could not finish");
                ui.add_space(12.0);
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        ui.label(error);
                    });
                ui.add_space(14.0);
                if ui.add(IconButton::new(Icon::Stop, "Close")).clicked() {
                    self.error = None;
                }
            });
        }
        if self.file_dialog.is_some() {
            let mut apply = false;
            let mut dismiss = false;
            if let Some(dialog) = &mut self.file_dialog {
                egui::Modal::new(egui::Id::unique("file-name")).show(ctx, |ui| {
                    ui.set_width(360.0);
                    ui.heading(if dialog.original.is_some() {
                        "Rename item"
                    } else {
                        "New folder"
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.name)
                            .hint_text("Name")
                            .desired_width(f32::INFINITY),
                    );
                    ui.horizontal(|ui| {
                        dismiss = ui.add(IconButton::new(Icon::Stop, "Cancel")).clicked();
                        apply = ui
                            .add_enabled(
                                !dialog.name.trim().is_empty(),
                                egui::Button::new("Continue"),
                            )
                            .clicked();
                    });
                });
            }
            if apply {
                if let Some(dialog) = self.file_dialog.take() {
                    if dialog.name.contains('/') || dialog.name == "." || dialog.name == ".." {
                        self.error =
                            Some("Enter a name without path separators or traversal.".into());
                    } else if let Some(from) = dialog.original {
                        let parent = from.rsplit_once('/').map(|(p, _)| p).unwrap_or("/sdcard");
                        self.action(Action::Rename {
                            to: format!("{parent}/{}", dialog.name),
                            from,
                        });
                    } else {
                        self.action(Action::CreateDirectory(format!(
                            "{}/{}",
                            self.path.trim_end_matches('/'),
                            dialog.name
                        )));
                    }
                }
            } else if dismiss {
                self.file_dialog = None;
            }
        }
        if let Some(texture) = self.preview.clone() {
            let mut open = true;
            egui::Window::new("Photo preview")
                .open(&mut open)
                .default_size([800.0, 600.0])
                .show(ctx, |ui| {
                    ui.label(self.preview_path.rsplit('/').next().unwrap_or("Photo"));
                    if ui
                        .add_enabled(
                            !self.busy,
                            IconButton::new(Icon::Download, "Download photo"),
                        )
                        .clicked()
                    {
                        self.submit(
                            Task::Choose(Choose::Download(self.preview_path.clone())),
                            "Choose download location",
                        );
                    }
                    ui.add(
                        egui::Image::new(&texture)
                            .max_size(ui.available_size())
                            .maintain_aspect_ratio(true),
                    );
                });
            if !open {
                self.preview = None;
            }
        }
    }
}

impl eframe::App for DroidOps {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.receive(&ctx);
        egui::Panel::bottom("status")
            .exact_size(38.0)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::Margin::symmetric(18, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.busy {
                        ui.spinner();
                    } else {
                        icons::show(ui, Icon::Check, ACCENT);
                    }
                    ui.label(&self.status);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.mirroring
                            && ui
                                .add(IconButton::new(Icon::Stop, "Stop mirroring"))
                                .clicked()
                        {
                            self.worker.stop_mirror();
                        }
                        if self.busy
                            && ui
                                .add(IconButton::new(Icon::Stop, "Cancel operation"))
                                .clicked()
                        {
                            self.worker.cancel();
                        }
                        ui.weak(format!(
                            "{} device{}",
                            self.devices.len(),
                            if self.devices.len() == 1 { "" } else { "s" }
                        ));
                    });
                });
            });
        egui::Panel::top("brand")
            .exact_size(52.0)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::Margin::symmetric(20, 12)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("DROIDOPS").strong().size(24.0));
                    ui.add_space(18.0);
                    ui.separator();
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("ANDROID DEVICE WORKSPACE")
                            .monospace()
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(IconButton::new(Icon::Settings, "Appearance"))
                            .clicked()
                        {
                            self.screen = Screen::Settings;
                        }
                        ui.label(
                            RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .monospace()
                                .size(10.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                });
            });
        egui::Panel::top("navigation")
            .exact_size(47.0)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::Margin::symmetric(18, 6)),
            )
            .show(ui, |ui| self.navigation(ui));
        egui::Panel::top("toolbar")
            .exact_size(62.0)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(egui::Margin::symmetric(24, 16)),
            )
            .show(ui, |ui| self.header(ui));
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(26),
            )
            .show(ui, |ui| {
                self.content(ui);
            });
        self.dialogs(&ctx);
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !self.busy && self.screen == Screen::Applications {
            if let Some(path) = dropped.first().map(|file| file.path().to_path_buf()) {
                self.action(Action::Install(path));
            }
        } else if !self.busy && self.screen == Screen::Files {
            if let Some(local) = dropped.first().map(|file| file.path().to_path_buf()) {
                if let Some(name) = local.file_name().and_then(|s| s.to_str()) {
                    let remote = format!("{}/{}", self.path.trim_end_matches('/'), name);
                    self.action(Action::Push { local, remote });
                }
            }
        }
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "preferences", &self.preferences);
    }
}

fn theme(ctx: &egui::Context, preferences: &Preferences) {
    let mut visuals = if preferences.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    if preferences.dark {
        visuals.panel_fill = Color32::from_rgb(24, 25, 27);
        visuals.window_fill = Color32::from_rgb(29, 30, 32);
        visuals.extreme_bg_color = Color32::from_rgb(18, 19, 21);
    } else {
        visuals.panel_fill = Color32::from_rgb(243, 242, 239);
        visuals.window_fill = Color32::from_rgb(243, 242, 239);
        visuals.extreme_bg_color = Color32::from_rgb(235, 234, 231);
        visuals.override_text_color = Some(Color32::from_rgb(22, 22, 23));
        visuals.weak_text_color = Some(Color32::from_rgb(104, 104, 104));
    }
    visuals.selection.bg_fill = if preferences.dark {
        Color32::from_rgb(28, 47, 76)
    } else {
        Color32::from_rgb(223, 232, 246)
    };
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    visuals.window_corner_radius = egui::CornerRadius::ZERO;
    visuals.window_shadow = egui::epaint::Shadow::NONE;
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.noninteractive,
    ] {
        widget.corner_radius = egui::CornerRadius::ZERO;
        widget.bg_stroke = egui::Stroke::new(
            1.0,
            if preferences.dark {
                Color32::from_gray(65)
            } else {
                Color32::from_rgb(206, 205, 201)
            },
        );
    }
    if !preferences.dark {
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(239, 238, 234);
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(227, 226, 222);
    }
    ctx.set_visuals(visuals);
    ctx.set_zoom_factor(preferences.scale);
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = Vec2::new(10.0, 9.0);
        style.spacing.button_padding = Vec2::new(12.0, 8.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
        style
            .text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(11.0));
    });
}

fn rule(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_gray(63)
    } else {
        Color32::from_rgb(203, 202, 198)
    }
}
fn card(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, rule(ui)))
        .inner_margin(20)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                contents(ui);
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn tool_tile(
    ui: &mut egui::Ui,
    width: f32,
    index: usize,
    title: &str,
    subtitle: &str,
    description: &str,
    meta: &str,
    cta: &str,
    selected: bool,
) -> bool {
    let mut clicked = false;
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, rule(ui)))
        .inner_margin(20)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_min_size(Vec2::new(width - 42.0, 228.0));
                ui.set_max_width(width - 42.0);
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(38.0), egui::Sense::hover());
                    tool_icon(ui, rect, index);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(title).size(21.0).strong());
                        ui.weak(subtitle);
                    });
                });
                ui.add_space(16.0);
                badge(
                    ui,
                    if selected {
                        "READY TO OPEN"
                    } else {
                        "DEVICE REQUIRED"
                    },
                );
                ui.add_space(12.0);
                ui.label(description);
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(meta)
                                .monospace()
                                .size(10.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                    ui.separator();
                    clicked = ui.add_sized([width - 42.0, 34.0], primary(cta)).clicked();
                });
            });
        });
    clicked
}

fn primary(text: &str) -> egui::Button<'_> {
    egui::Button::new(
        RichText::new(text)
            .monospace()
            .size(11.0)
            .strong()
            .color(Color32::WHITE),
    )
    .fill(Color32::from_rgb(20, 20, 22))
    .corner_radius(0)
}
fn badge(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, rule(ui)))
        .inner_margin(egui::Margin::symmetric(5, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).monospace().size(10.0).color(ACCENT));
        });
}
fn tool_icon(ui: &egui::Ui, rect: egui::Rect, index: usize) {
    let icon = [
        Icon::Apps,
        Icon::Folder,
        Icon::Image,
        Icon::Terminal,
        Icon::Recovery,
        Icon::Power,
    ][index.min(5)];
    icons::paint(ui, rect.shrink(3.0), icon, ui.visuals().text_color());
}
fn screen_icon(screen: Screen) -> Icon {
    match screen {
        Screen::Dashboard => Icon::Dashboard,
        Screen::Applications => Icon::Apps,
        Screen::Files => Icon::Folder,
        Screen::Gallery => Icon::Image,
        Screen::Shell => Icon::Terminal,
        Screen::Recovery => Icon::Recovery,
        Screen::Fastboot => Icon::Power,
        Screen::Settings => Icon::Settings,
    }
}
fn is_image(name: &str) -> bool {
    name.rsplit('.')
        .next()
        .is_some_and(|ext| ["jpg", "jpeg", "png", "webp"].contains(&ext.to_lowercase().as_str()))
}
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    }
}
fn append_output(output: &mut String, text: &str) {
    output.push_str(text);
    output.push('\n');
    if output.len() > 128 * 1024 {
        let mut start = output.len() - 128 * 1024;
        while !output.is_char_boundary(start) {
            start += 1;
        }
        output.drain(..start);
    }
}

#[cfg(test)]
mod ui_tests {
    use super::*;

    #[test]
    fn every_screen_renders_with_fixtures_at_standard_and_large_scales() {
        for scale in [1.0, 1.5] {
            let ctx = egui::Context::default();
            let preferences = Preferences { dark: false, scale };
            theme(&ctx, &preferences);
            let mut app = DroidOps::with_worker(Worker::offline(), preferences);
            app.session.select(Some(Device {
                serial: "fixture".into(),
                model: "Test device".into(),
                state: "device".into(),
                transport: Transport::Adb,
            }));
            app.info = Some(DeviceInfo {
                model: "Test device".into(),
                android: "16".into(),
                battery: "72".into(),
                charging: "Charging".into(),
            });
            app.apps = vec!["com.example.app".into()];
            app.files = vec![FileEntry {
                path: "/sdcard/photo.png".into(),
                name: "photo.png".into(),
                directory: false,
                size: 1024,
            }];
            app.media = vec!["/sdcard/DCIM/photo.png".into()];
            for screen in Screen::ALL {
                app.screen = screen;
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1100.0, 850.0),
                    )),
                    ..Default::default()
                };
                let mut warmup = ctx.run_ui(input.clone(), |_| {});
                warmup.textures_delta.clear();
                let mut result = ctx.run_ui(input, |ui| {
                    app.content(ui);
                    assert!(
                        ui.min_rect().width() <= 1101.0,
                        "screen exceeds viewport: {screen:?}, width={}, scale={scale}",
                        ui.min_rect().width()
                    );
                });
                result.textures_delta.clear();
                assert!(
                    !result.shapes.is_empty(),
                    "screen has no visible content: {screen:?}"
                );
            }
        }
    }

    #[test]
    fn all_icons_render_without_font_glyphs() {
        let ctx = egui::Context::default();
        let mut result = ctx.run_ui(egui::RawInput::default(), |ui| {
            for icon in [
                Icon::Dashboard,
                Icon::Apps,
                Icon::Folder,
                Icon::Image,
                Icon::Terminal,
                Icon::Recovery,
                Icon::Power,
                Icon::Settings,
                Icon::Refresh,
                Icon::Device,
                Icon::File,
                Icon::Upload,
                Icon::Download,
                Icon::Delete,
                Icon::Rename,
                Icon::Clear,
                Icon::Plus,
                Icon::Up,
                Icon::Stop,
                Icon::Check,
            ] {
                icons::show(ui, icon, Color32::BLACK);
            }
        });
        result.textures_delta.clear();
        assert!(!result.shapes.is_empty());
        assert!(result
            .shapes
            .iter()
            .all(|shape| !matches!(shape.shape, egui::Shape::Text(_))));
    }
}
