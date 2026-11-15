//! Native application screens; services execute only through background requests.

use super::*;

impl DroidOps {
    pub(super) fn content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(5.0);
        ui.label(RichText::new(self.screen.title()).size(29.0).strong());
        ui.weak(self.screen.subtitle());
        ui.add_space(22.0);
        if self.screen == Screen::Settings {
            self.settings(ui);
            return;
        }
        if self.screen == Screen::Applications {
            self.active_app_windows(ui);
        }
        if self.session.device.is_none() && self.screen != Screen::Dashboard {
            self.empty(ui);
            return;
        }
        if self.session.device.as_ref().is_some_and(|d| !d.online())
            && !matches!(
                self.screen,
                Screen::Fastboot | Screen::Recovery | Screen::Dashboard
            )
        {
            card(ui, |ui| {
                ui.label("This screen needs an authorized Android device in normal ADB mode.");
                ui.weak("Check the device's USB debugging prompt, or return it to Android.");
            });
            return;
        }
        match self.screen {
            Screen::Dashboard => self.dashboard(ui),
            Screen::Applications => self.applications(ui),
            Screen::Files => self.file_explorer(ui),
            Screen::Gallery => self.gallery(ui),
            Screen::Shell => self.shell_ui(ui),
            Screen::Recovery => self.recovery(ui),
            Screen::Fastboot => self.power(ui),
            Screen::Settings => {}
        }
    }

    fn empty(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.set_min_height(260.0);
            ui.add_space(20.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(76.0, 92.0), egui::Sense::hover());
            let phone =
                egui::Rect::from_min_size(rect.min + Vec2::new(12.0, 0.0), Vec2::new(45.0, 76.0));
            ui.painter().rect_stroke(
                phone,
                8.0,
                egui::Stroke::new(2.0, ACCENT),
                egui::StrokeKind::Inside,
            );
            ui.painter().line_segment(
                [
                    phone.center_top() + Vec2::new(-7.0, 8.0),
                    phone.center_top() + Vec2::new(7.0, 8.0),
                ],
                egui::Stroke::new(2.0, ACCENT),
            );
            ui.label(
                RichText::new(if self.devices.is_empty() {
                    "Connect your Android device"
                } else {
                    "Choose your device"
                })
                .size(22.0)
                .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                "Connect with USB, enable USB debugging, then allow this computer on your phone.",
            );
            ui.weak("Select the device in the top bar. DroidOps never chooses a target for you.");
            ui.add_space(18.0);
            if ui
                .add(IconButton::new(Icon::Refresh, "Look for devices"))
                .clicked()
            {
                self.worker.refresh();
            }
        });
        ui.add_space(18.0);
        ui.horizontal_wrapped(|ui| {
            ui.weak("ADB and Fastboot must be on PATH.");
            ui.hyperlink_to(
                "Platform tools",
                "https://developer.android.com/tools/releases/platform-tools",
            );
        });
        if !self.warnings.is_empty() {
            ui.add_space(12.0);
            for warning in &self.warnings {
                ui.colored_label(ACCENT, warning);
            }
        }
    }

    fn dashboard(&mut self, ui: &mut egui::Ui) {
        if let Some(device) = self.session.device.clone() {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&device.model).size(22.0).strong());
                    badge(ui, &device.state.to_uppercase());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                device.online() && !self.busy && !self.mirroring,
                                primary("MIRROR SCREEN"),
                            )
                            .clicked()
                        {
                            self.action(Action::Mirror);
                        }
                        if ui
                            .add_enabled(
                                device.online() && !self.busy,
                                IconButton::new(Icon::Refresh, "Refresh"),
                            )
                            .clicked()
                        {
                            self.reload = true;
                        }
                    });
                });
                ui.weak(format!("{} · {:?}", device.serial, device.transport));
                if let Some(info) = &self.info {
                    ui.add_space(12.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("Android {}", info.android));
                        ui.separator();
                        ui.label(format!("Battery {}%", info.battery));
                        ui.separator();
                        ui.label(&info.charging);
                    });
                }
            });
        } else {
            ui.horizontal_wrapped(|ui| {
                badge(ui, if self.devices.is_empty(){"AWAITING CONNECTION"}else{"CHOOSE A DEVICE"});
                ui.label("Connect over USB, enable USB debugging, and allow this computer on your phone.");
            });
        }
        ui.add_space(22.0);
        let tools = [
            (
                Screen::Applications,
                "Applications",
                "Android package management",
                "List installed packages, install APKs, or manage app data.",
                "ADB · PACKAGES",
                "OPEN APPLICATIONS",
            ),
            (
                Screen::Files,
                "File explorer",
                "Your shared storage",
                "Browse folders, upload files, and download content to your computer.",
                "ADB · STORAGE",
                "BROWSE FILES",
            ),
            (
                Screen::Gallery,
                "Gallery",
                "Photos, close at hand",
                "Find pictures by album, open a full preview, and save a local copy.",
                "DCIM · PICTURES",
                "OPEN GALLERY",
            ),
            (
                Screen::Shell,
                "ADB shell",
                "Direct device commands",
                "Run a command with explicit confirmation and inspect the output.",
                "ADB · COMMANDS",
                "OPEN SHELL",
            ),
            (
                Screen::Recovery,
                "Recovery sideload",
                "Android recovery updates",
                "Select a ZIP update for a device in Apply update from ADB mode.",
                "ADB · RECOVERY",
                "OPEN RECOVERY",
            ),
            (
                Screen::Fastboot,
                "Power & Fastboot",
                "The right device mode",
                "Restart Android or switch to bootloader and recovery modes.",
                "ADB · FASTBOOT",
                "OPEN POWER TOOLS",
            ),
        ];
        let columns = if ui.available_width() > 1050.0 {
            3
        } else if ui.available_width() > 650.0 {
            2
        } else {
            1
        };
        let width = (ui.available_width() - 18.0) / columns as f32;
        egui::ScrollArea::vertical()
            .id_salt("workspace-tools")
            .show(ui, |ui| {
                egui::Grid::new("tools-grid")
                    .num_columns(columns)
                    .spacing(Vec2::ZERO)
                    .show(ui, |ui| {
                        for (index, (screen, title, subtitle, description, meta, cta)) in
                            tools.iter().enumerate()
                        {
                            if tool_tile(
                                ui,
                                width,
                                index,
                                title,
                                subtitle,
                                description,
                                meta,
                                cta,
                                self.session.device.is_some(),
                            ) {
                                self.screen = *screen;
                                self.reload = true;
                            }
                            if (index + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
    }

    fn applications(&mut self, ui: &mut egui::Ui) {
        self.app_window_options(ui);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("Search package name")
                    .desired_width(250.0),
            );
            let old = self.app_filter.clone();
            egui::ComboBox::from_id_salt("package-filter")
                .selected_text(&self.app_filter)
                .show_ui(ui, |ui| {
                    for filter in ["User", "System", "All"] {
                        ui.selectable_value(&mut self.app_filter, filter.into(), filter);
                    }
                });
            if self.app_filter != old {
                self.apps.clear();
                self.reload = true;
            }
            if ui
                .add_enabled(!self.busy, IconButton::new(Icon::Refresh, "Refresh"))
                .clicked()
            {
                self.reload = true;
            }
            if ui.add_enabled(!self.busy, primary("INSTALL APK")).clicked() {
                self.submit(Task::Choose(Choose::Install), "Choose an APK");
            }
        });
        ui.add_space(14.0);
        let query = self.search.to_lowercase();
        let apps: Vec<_> = self
            .apps
            .iter()
            .filter(|a| {
                a.to_lowercase().contains(&query)
                    || self
                        .app_names
                        .get(*a)
                        .is_some_and(|n| n.to_lowercase().contains(&query))
            })
            .cloned()
            .collect();
        ui.weak(format!("{} packages", apps.len()));
        ui.add_space(8.0);
        egui::ScrollArea::vertical().id_salt("packages").show_rows(
            ui,
            44.0,
            apps.len(),
            |ui, rows| {
                for index in rows {
                    let package = &apps[index];
                    ui.horizontal(|ui| {
                        icons::show(ui, Icon::Apps, ui.visuals().text_color());
                        let name = self
                            .app_names
                            .get(package)
                            .cloned()
                            .unwrap_or_else(|| package.clone());
                        ui.vertical(|ui| {
                            ui.label(&name);
                            if name != *package {
                                ui.weak(package);
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(!self.busy, egui::Button::new("Open as Desktop App"))
                                .clicked()
                            {
                                let title = if self.app_window_title.trim().is_empty() {
                                    name.clone()
                                } else {
                                    self.app_window_title.trim().to_owned()
                                };
                                self.launch_app_window(package.clone(), title, false);
                            }
                            if ui
                                .add_enabled(!self.busy, IconButton::new(Icon::Delete, "Uninstall"))
                                .clicked()
                            {
                                self.action(Action::Uninstall(package.clone()));
                            }
                            if ui
                                .add_enabled(!self.busy, IconButton::new(Icon::Clear, "Clear data"))
                                .clicked()
                            {
                                self.action(Action::ClearData(package.clone()));
                            }
                        });
                    });
                    ui.separator();
                }
            },
        );
        if self.apps.is_empty() && !self.busy {
            ui.weak("No packages loaded. Choose a filter and refresh.");
        }
    }

    fn file_explorer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.busy, IconButton::new(Icon::Up, "Up"))
                .clicked()
            {
                let trimmed = self.path.trim_end_matches('/');
                let target = trimmed
                    .rsplit_once('/')
                    .map(|(parent, _)| {
                        if parent.is_empty() {
                            "/".into()
                        } else {
                            parent.into()
                        }
                    })
                    .unwrap_or_else(|| "/".into());
                self.submit(Task::Files(target), "Reading directory");
            }
            let path = ui.add_enabled(
                !self.busy,
                egui::TextEdit::singleline(&mut self.path_draft).desired_width(300.0),
            );
            if !self.busy
                && ((path.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || ui.add(IconButton::new(Icon::Folder, "Go")).clicked())
            {
                self.submit(Task::Files(self.path_draft.clone()), "Reading directory");
            }
        });
        ui.weak(format!("Showing: {}", self.path));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.busy, IconButton::new(Icon::Upload, "Upload file"))
                .clicked()
            {
                self.submit(
                    Task::Choose(Choose::Upload(self.path.clone())),
                    "Choose a file to upload",
                );
            }
            if ui
                .add_enabled(!self.busy, IconButton::new(Icon::Plus, "New folder"))
                .clicked()
            {
                self.file_dialog = Some(FileDialog {
                    original: None,
                    name: String::new(),
                });
            }
            if ui
                .add_enabled(!self.busy, IconButton::new(Icon::Refresh, "Refresh"))
                .clicked()
            {
                self.reload = true;
            }
            ui.weak(format!("{} items", self.files.len()));
        });
        ui.add_space(12.0);
        let files = self.files.clone();
        egui::ScrollArea::vertical().id_salt("files").show_rows(
            ui,
            42.0,
            files.len(),
            |ui, rows| {
                for index in rows {
                    let file = &files[index];
                    ui.horizontal(|ui| {
                        icons::show(
                            ui,
                            if file.directory {
                                Icon::Folder
                            } else if is_image(&file.name) {
                                Icon::Image
                            } else {
                                Icon::File
                            },
                            ui.visuals().text_color(),
                        );
                        if ui
                            .add_enabled(!self.busy, egui::Button::new(&file.name).frame(false))
                            .clicked()
                        {
                            if file.directory {
                                self.submit(Task::Files(file.path.clone()), "Reading directory");
                            } else if is_image(&file.name) {
                                self.preview_path = file.path.clone();
                                self.submit(Task::Preview(file.path.clone()), "Loading photo");
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(!self.busy, IconButton::new(Icon::Delete, "Delete"))
                                .clicked()
                            {
                                self.action(Action::Delete(file.path.clone()));
                            }
                            if ui
                                .add_enabled(!self.busy, IconButton::new(Icon::Rename, "Rename"))
                                .clicked()
                            {
                                self.file_dialog = Some(FileDialog {
                                    original: Some(file.path.clone()),
                                    name: file.name.clone(),
                                });
                            }
                            if ui
                                .add_enabled(
                                    !self.busy,
                                    IconButton::new(Icon::Download, "Download"),
                                )
                                .clicked()
                            {
                                self.submit(
                                    Task::Choose(Choose::Download(file.path.clone())),
                                    "Choose download location",
                                );
                            }
                            if !file.directory {
                                ui.weak(format_bytes(file.size));
                            }
                        });
                    });
                    ui.separator();
                }
            },
        );
    }

    fn gallery(&mut self, ui: &mut egui::Ui) {
        let mut albums: Vec<String> = self
            .media
            .iter()
            .filter_map(|path| {
                path.rsplit_once('/')
                    .map(|(p, _)| p.rsplit('/').next().unwrap_or(p).to_owned())
            })
            .collect();
        albums.sort();
        albums.dedup();
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("albums")
                .selected_text(&self.album)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.album, "All albums".into(), "All albums");
                    for album in &albums {
                        ui.selectable_value(&mut self.album, album.clone(), album);
                    }
                });
            if ui
                .add_enabled(!self.busy, IconButton::new(Icon::Refresh, "Refresh photos"))
                .clicked()
            {
                self.reload = true;
            }
            ui.weak(format!("{} photos", self.media.len()));
        });
        ui.add_space(12.0);
        let paths: Vec<String> = self
            .media
            .iter()
            .filter(|path| {
                self.album == "All albums"
                    || path
                        .rsplit_once('/')
                        .is_some_and(|(p, _)| p.rsplit('/').next() == Some(self.album.as_str()))
            })
            .cloned()
            .collect();
        egui::ScrollArea::vertical().id_salt("photos").show_rows(
            ui,
            46.0,
            paths.len(),
            |ui, rows| {
                for index in rows {
                    let path = &paths[index];
                    ui.horizontal(|ui| {
                        icons::show(ui, Icon::Image, ui.visuals().text_color());
                        if ui
                            .add_enabled(
                                !self.busy,
                                egui::Button::new(path.rsplit('/').next().unwrap_or(path))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            self.preview_path = path.clone();
                            self.submit(Task::Preview(path.clone()), "Loading photo");
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(
                                    !self.busy,
                                    IconButton::new(Icon::Download, "Download"),
                                )
                                .clicked()
                            {
                                self.submit(
                                    Task::Choose(Choose::Download(path.clone())),
                                    "Choose download location",
                                );
                            }
                        });
                    });
                    ui.separator();
                }
            },
        );
        if self.media.is_empty() && !self.busy {
            ui.weak("No supported photos found in DCIM or Pictures. Refresh after taking a photo.");
        }
    }

    fn shell_ui(&mut self, ui: &mut egui::Ui) {
        ui.weak("Commands run on your phone and may change or delete data. Each command requires confirmation.");
        ui.add_space(10.0);
        egui::ScrollArea::vertical()
            .max_height((ui.available_height() - 145.0).max(100.0))
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.output)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(16)
                        .interactive(false),
                );
            });
        ui.add_space(10.0);
        ui.add(
            egui::TextEdit::multiline(&mut self.shell)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(2)
                .hint_text("Example: getprop ro.product.model"),
        );
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !self.busy && !self.shell.trim().is_empty(),
                    primary("RUN COMMAND"),
                )
                .clicked()
            {
                self.action(Action::Shell(self.shell.clone()));
            }
            if ui.button("Clear output").clicked() {
                self.output.clear();
            }
        });
    }

    fn recovery(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.label(RichText::new("Recovery ZIP update").size(22.0).strong());
            ui.add_space(10.0);
            ui.label("Use this for an OTA or recovery ZIP. For APK files, open Applications.");
            ui.label("Open Android recovery and select Apply update from ADB, then select the sideload device in the top bar.");
            ui.add_space(16.0);
            let ready = self
                .session
                .device
                .as_ref()
                .is_some_and(|d| d.transport == Transport::Adb && d.state == "sideload");
            if ui
                .add_enabled(ready && !self.busy, primary("CHOOSE RECOVERY ZIP"))
                .clicked()
            {
                self.submit(Task::Choose(Choose::Sideload), "Choose recovery ZIP");
            }
            if !ready {
                ui.weak("Waiting for a selected device in sideload mode.");
            }
        });
    }

    fn power(&mut self, ui: &mut egui::Ui) {
        let Some(device) = self.session.device.clone() else {
            return;
        };
        let boot = device.transport == Transport::Fastboot;
        card(ui, |ui| {
            ui.label(
                RichText::new(if boot {
                    "Fastboot mode"
                } else {
                    "Restart your device"
                })
                .size(22.0)
                .strong(),
            );
            ui.weak("Restarting interrupts applications and current connections.");
            ui.add_space(18.0);
            ui.add_enabled_ui(!self.busy && (boot || device.online()), |ui| {
                if ui.button("Reboot to system").clicked() {
                    self.action(Action::Reboot("".into()));
                }
                if !boot {
                    ui.add_space(10.0);
                    if ui.button("Reboot to bootloader").clicked() {
                        self.action(Action::Reboot("bootloader".into()));
                    }
                    if ui.button("Reboot to recovery").clicked() {
                        self.action(Action::Reboot("recovery".into()));
                    }
                }
            });
            ui.add_space(18.0);
            ui.weak("DroidOps does not expose flashing, erasing, or bootloader unlocking.");
        });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        card(ui, |ui| {
            ui.label(RichText::new("Appearance").size(20.0).strong());
            ui.add_space(12.0);
            let dark = ui
                .checkbox(&mut self.preferences.dark, "Dark theme")
                .changed();
            let scale = ui
                .add(
                    egui::Slider::new(&mut self.preferences.scale, 0.8..=1.6)
                        .text("Interface scale"),
                )
                .changed();
            if dark || scale {
                theme(ui.ctx(), &self.preferences);
            }
            ui.add_space(14.0);
            ui.weak("Appearance is saved on this computer. Device selection and shell commands are never restored.");
        });
        ui.add_space(18.0);
        card(ui, |ui| {
            ui.label(RichText::new("Device tools").size(20.0).strong());
            ui.add_space(10.0);
            ui.label("ADB and Fastboot are discovered from PATH. Mirroring also needs scrcpy.");
            ui.label("Install tools with your operating system's package manager or Android SDK.");
            ui.hyperlink_to(
                "Android platform tools",
                "https://developer.android.com/tools/releases/platform-tools",
            );
            if ui.button("Check device tools").clicked() {
                self.worker.refresh();
            }
            for warning in &self.warnings {
                ui.colored_label(ACCENT, warning);
            }
        });
    }
}
