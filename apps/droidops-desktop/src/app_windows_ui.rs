//! Application-window controls dispatch commands to the owned background manager.
use super::*;
use droidops_device_services::app_windows::AppWindowStatus;
impl DroidOps {
    pub(super) fn active_app_windows(&mut self, ui: &mut egui::Ui) {
        if self.app_windows.is_empty() {
            return;
        }
        ui.label(RichText::new("Active App Windows").strong());
        egui::ScrollArea::vertical()
            .id_salt("active-app-windows")
            .max_height(150.0)
            .show(ui, |ui| {
                for session in &self.app_windows {
                    ui.horizontal_wrapped(|ui| {
                        icons::show(ui, Icon::Apps, ui.visuals().text_color());
                        ui.label(format!(
                            "{} · {} · {}",
                            session.title, session.device.model, session.device.serial
                        ));
                        let status = match &session.status {
                            AppWindowStatus::Starting => "Starting; verifying app display".into(),
                            AppWindowStatus::Running => "Running".into(),
                            AppWindowStatus::Stopping => "Stopping".into(),
                            AppWindowStatus::Closed => "Closed".into(),
                            AppWindowStatus::Failed(e) => format!("Failed: {e}"),
                        };
                        ui.weak(status);
                        if session.status.active()
                            && ui.add(IconButton::new(Icon::Stop, "Stop")).clicked()
                        {
                            if let Err(e) = self.worker.app_command(AppCommand::Stop(session.id)) {
                                self.error = Some(e);
                            }
                        }
                    });
                }
            });
        ui.separator();
    }
    pub(super) fn app_window_options(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Desktop app window settings").show(ui, |ui| {
            let settings = &mut self.preferences.app_windows;
            ui.checkbox(&mut settings.match_phone,"Match phone resolution and DPI at launch");
            ui.add_enabled_ui(!settings.match_phone,|ui| {ui.horizontal_wrapped(|ui| {
                ui.label("Virtual width");
                ui.add(egui::DragValue::new(&mut settings.width).range(320..=4096));
                ui.label("Height");
                ui.add(egui::DragValue::new(&mut settings.height).range(320..=4096));
                ui.label("DPI");
                ui.add(egui::DragValue::new(&mut settings.dpi).range(72..=640));
            });});
            ui.horizontal(|ui| {
                ui.label("FPS");
                ui.add(egui::DragValue::new(&mut settings.fps).range(1..=120));
            });
            ui.horizontal_wrapped(|ui| {
                let mut live_resolution=settings.flex && !settings.match_phone;
                if ui.add_enabled(!settings.match_phone,egui::Checkbox::new(&mut live_resolution,"Live Android resolution (custom mode)")).changed(){settings.flex=live_resolution;}
                ui.checkbox(&mut settings.audio, "Application audio when permitted");
                ui.checkbox(&mut settings.decorations, "Android system decorations");
                ui.checkbox(&mut settings.preserve_content, "Move content to phone when closing");
            });
            ui.add_enabled_ui(!settings.flex || settings.match_phone, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Desktop width");
                    ui.add(egui::DragValue::new(&mut settings.window_width).range(200..=4096));
                    ui.label("Desktop height");
                    ui.add(egui::DragValue::new(&mut settings.window_height).range(200..=4096));
                });
            });
            ui.weak("Phone mode preserves resolution and DPI and scales the whole picture as you resize. Disable phone matching for live Android-resolution changes; high DPI in small windows can make app controls too large.");
            ui.checkbox(&mut settings.force_stop, "Force-stop before launch (requires confirmation)");
            ui.horizontal(|ui| {
                ui.label("Window title (blank uses app name)");
                ui.text_edit_singleline(&mut self.app_window_title);
            });
            ui.weak("Apps run on your phone. Closing DroidOps stops all owned windows. Closing a display normally destroys its content; moving content to the phone may interrupt its primary screen.");
            ui.weak("Protected content and incompatible apps may not work. Clipboard uses scrcpy defaults and is shared with the phone. Audio capture can include other apps.");
        });
        if let Some((generation, package, title, settings)) = self.pending_app_window.clone() {
            if !self.session.accepts(generation) {
                self.pending_app_window = None;
                return;
            }
            card(ui, |ui| {
                let device = self
                    .session
                    .device
                    .as_ref()
                    .map(|d| d.serial.as_str())
                    .unwrap_or("Unavailable");
                ui.label(format!("Force-stop {package} on {device} before opening {title}? This interrupts any existing phone session for this app."));
                ui.horizontal(|ui| {
                    if ui.button("Confirm force-stop and launch").clicked() {
                        self.pending_app_window = None;
                        if let Some(device) = self.session.device.clone().filter(Device::online) {
                            if let Err(e) = self.worker.app_command(AppCommand::Launch(
                                device,
                                package.clone(),
                                title.clone(),
                                settings.clone(),
                                true,
                            )) {
                                self.error = Some(e);
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        self.pending_app_window = None;
                    }
                });
            });
        }
    }
    pub(super) fn launch_app_window(&mut self, package: String, title: String, confirmed: bool) {
        let Some(device) = self.session.device.clone().filter(Device::online) else {
            self.error = Some("Select an authorized Android device first".into());
            return;
        };
        if self.preferences.app_windows.force_stop && !confirmed {
            self.pending_app_window = Some((
                self.session.generation,
                package,
                title,
                self.preferences.app_windows.clone(),
            ));
            return;
        }
        if let Err(e) = self.worker.app_command(AppCommand::Launch(
            device,
            package,
            title,
            self.preferences.app_windows.clone(),
            confirmed,
        )) {
            self.error = Some(e);
        }
    }
}
