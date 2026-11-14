//! Font-independent outline icons on a shared 24-unit grid.

use eframe::egui::{self, Color32, Pos2, Rect, Response, Stroke, Vec2, Widget};

#[derive(Clone, Copy)]
pub enum Icon {
    Dashboard,
    Apps,
    Folder,
    Image,
    Terminal,
    Recovery,
    Power,
    Settings,
    Refresh,
    Device,
    File,
    Upload,
    Download,
    Delete,
    Rename,
    Clear,
    Plus,
    Up,
    Stop,
    Check,
}

/// Use ordinary egui buttons for focus, keyboard activation, tooltips, and labels.
pub struct IconButton<'a> {
    icon: Icon,
    label: &'a str,
    selected: bool,
}
impl<'a> IconButton<'a> {
    pub fn new(icon: Icon, label: &'a str) -> Self {
        Self {
            icon,
            label,
            selected: false,
        }
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}
impl Widget for IconButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let text = format!("      {}", self.label);
        let response = egui::Button::new(text)
            .selected(self.selected)
            .corner_radius(0)
            .ui(ui);
        let center = Pos2::new(response.rect.left() + 17.0, response.rect.center().y);
        let color = if self.selected {
            super::ACCENT
        } else {
            ui.style().interact(&response).fg_stroke.color
        };
        paint(
            ui,
            Rect::from_center_size(center, Vec2::splat(17.0)),
            self.icon,
            color,
        );
        response.on_hover_text(self.label)
    }
}

pub fn show(ui: &mut egui::Ui, icon: Icon, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), egui::Sense::hover());
    paint(ui, rect, icon, color);
}

pub fn paint(ui: &egui::Ui, rect: Rect, icon: Icon, color: Color32) {
    let p = ui.painter();
    let scale = rect.width().min(rect.height()) / 24.0;
    let point = |x: f32, y: f32| rect.min + Vec2::new(x, y) * scale;
    let stroke = Stroke::new((1.6 * scale).max(1.0), color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([point(a.0, a.1), point(b.0, b.1)], stroke);
    };
    let path = |coords: &[(f32, f32)]| {
        p.add(egui::Shape::line(
            coords.iter().map(|&(x, y)| point(x, y)).collect(),
            stroke,
        ));
    };
    let box_outline = |x: f32, y: f32, w: f32, h: f32| {
        p.rect_stroke(
            Rect::from_min_size(point(x, y), Vec2::new(w, h) * scale),
            0.0,
            stroke,
            egui::StrokeKind::Inside,
        );
    };
    match icon {
        Icon::Dashboard => {
            box_outline(3.0, 3.0, 7.0, 8.0);
            box_outline(14.0, 3.0, 7.0, 5.0);
            box_outline(3.0, 15.0, 7.0, 6.0);
            box_outline(14.0, 12.0, 7.0, 9.0);
        }
        Icon::Apps => {
            for y in [3.0, 14.0] {
                for x in [3.0, 14.0] {
                    box_outline(x, y, 7.0, 7.0);
                }
            }
        }
        Icon::Folder => path(&[
            (3.0, 7.0),
            (3.0, 20.0),
            (21.0, 20.0),
            (21.0, 7.0),
            (12.0, 7.0),
            (10.0, 4.0),
            (3.0, 4.0),
            (3.0, 7.0),
            (21.0, 7.0),
        ]),
        Icon::Image => {
            box_outline(3.0, 3.0, 18.0, 18.0);
            p.circle_stroke(point(8.0, 8.0), 2.0 * scale, stroke);
            path(&[
                (4.0, 19.0),
                (10.0, 13.0),
                (14.0, 17.0),
                (17.0, 14.0),
                (21.0, 18.0),
            ]);
        }
        Icon::Terminal => {
            path(&[(4.0, 5.0), (10.0, 11.0), (4.0, 17.0)]);
            line((13.0, 18.0), (21.0, 18.0));
        }
        Icon::Device | Icon::Recovery => {
            box_outline(6.0, 2.0, 12.0, 20.0);
            line((10.0, 5.0), (14.0, 5.0));
            line((11.0, 19.0), (13.0, 19.0));
            if matches!(icon, Icon::Recovery) {
                line((12.0, 8.0), (12.0, 15.0));
                path(&[(9.0, 12.0), (12.0, 15.0), (15.0, 12.0)]);
            }
        }
        Icon::Power => {
            let points = (0..=24)
                .map(|i| {
                    let a = std::f32::consts::PI / 3.0
                        + i as f32 * (5.0 * std::f32::consts::PI / 3.0) / 24.0;
                    point(12.0 + 8.0 * a.sin(), 12.0 - 8.0 * a.cos())
                })
                .collect();
            p.add(egui::Shape::line(points, stroke));
            line((12.0, 2.0), (12.0, 11.0));
        }
        Icon::Settings => {
            let points = (0..=32)
                .map(|i| {
                    let radius = if i % 4 < 2 { 10.0 } else { 7.6 };
                    let a = i as f32 * std::f32::consts::TAU / 32.0;
                    point(12.0 + radius * a.cos(), 12.0 + radius * a.sin())
                })
                .collect();
            p.add(egui::Shape::line(points, stroke));
            p.circle_stroke(point(12.0, 12.0), 3.0 * scale, stroke);
        }
        Icon::Refresh => {
            let points = (0..=20)
                .map(|i| {
                    let a = 0.3 + i as f32 * 5.0 / 20.0;
                    point(12.0 + 8.0 * a.cos(), 12.0 + 8.0 * a.sin())
                })
                .collect();
            p.add(egui::Shape::line(points, stroke));
            path(&[(15.0, 3.0), (17.0, 7.0), (21.0, 6.0)]);
        }
        Icon::File => path(&[
            (5.0, 2.0),
            (14.0, 2.0),
            (19.0, 7.0),
            (19.0, 22.0),
            (5.0, 22.0),
            (5.0, 2.0),
            (14.0, 2.0),
            (14.0, 7.0),
            (19.0, 7.0),
        ]),
        Icon::Upload | Icon::Download => {
            path(&[(3.0, 16.0), (3.0, 21.0), (21.0, 21.0), (21.0, 16.0)]);
            line((12.0, 3.0), (12.0, 16.0));
            if matches!(icon, Icon::Upload) {
                path(&[(7.0, 8.0), (12.0, 3.0), (17.0, 8.0)]);
            } else {
                path(&[(7.0, 11.0), (12.0, 16.0), (17.0, 11.0)]);
            }
        }
        Icon::Delete => {
            path(&[(6.0, 7.0), (7.0, 21.0), (17.0, 21.0), (18.0, 7.0)]);
            line((3.0, 5.0), (21.0, 5.0));
            path(&[(9.0, 5.0), (9.0, 2.0), (15.0, 2.0), (15.0, 5.0)]);
            line((10.0, 10.0), (10.0, 17.0));
            line((14.0, 10.0), (14.0, 17.0));
        }
        Icon::Rename => {
            path(&[
                (4.0, 20.0),
                (5.0, 14.0),
                (17.0, 2.0),
                (22.0, 7.0),
                (10.0, 19.0),
                (4.0, 20.0),
            ]);
            line((14.0, 5.0), (19.0, 10.0));
        }
        Icon::Clear => {
            path(&[
                (3.0, 15.0),
                (13.0, 3.0),
                (21.0, 10.0),
                (12.0, 21.0),
                (8.0, 21.0),
                (3.0, 15.0),
            ]);
            line((8.0, 10.0), (16.0, 17.0));
            line((12.0, 21.0), (22.0, 21.0));
        }
        Icon::Plus => {
            line((12.0, 4.0), (12.0, 20.0));
            line((4.0, 12.0), (20.0, 12.0));
        }
        Icon::Up => {
            line((12.0, 21.0), (12.0, 4.0));
            path(&[(5.0, 11.0), (12.0, 4.0), (19.0, 11.0)]);
        }
        Icon::Stop => box_outline(5.0, 5.0, 14.0, 14.0),
        Icon::Check => path(&[(4.0, 12.0), (9.0, 17.0), (20.0, 6.0)]),
    }
}

pub fn window_icon() -> egui::IconData {
    let mut rgba = vec![0; 64 * 64 * 4];
    for y in 0..64 {
        for x in 0..64 {
            let phone = (18..46).contains(&x) && (8..56).contains(&y);
            let screen = (21..43).contains(&x) && (14..50).contains(&y);
            let home = (29..35).contains(&x) && (52..54).contains(&y);
            let color = if phone && !screen && !home {
                [22, 22, 24, 255]
            } else {
                [243, 242, 239, 255]
            };
            rgba[(y * 64 + x) * 4..(y * 64 + x + 1) * 4].copy_from_slice(&color);
        }
    }
    egui::IconData {
        rgba,
        width: 64,
        height: 64,
    }
}
