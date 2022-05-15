use eframe::egui;
use egui::color::{Color32};
use egui::{Sense, Response, Vec2, Painter, Ui, Rect};

use leds;

fn main() {

    let options = eframe::NativeOptions::default();
    let app = MyApp::new("leds".to_string());
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(app)),
    );
}

struct MyApp {
    name: String,
    color: Color32,
}

impl MyApp {
    pub fn new(name: String) -> MyApp {
        MyApp {
            name,
            color: Color32::from_rgb(0xff, 0x99, 0x10)
        }
    }
}

// impl Default for MyApp {
//     fn default() -> Self {
//         Self {
//             name: "Arthur".to_owned(),
//             age: 42,
//             color: Color32::from_rgb(0xff, 0xba, 0x10),
//         }
//     }
// }

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            // ui.horizontal(|ui| {
            //     ui.label("Your name: ");
            //     ui.text_edit_singleline(&mut self.name);
            // });
            // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            // if ui.button("Click each year").clicked() {
            //     self.age += 1;
            // }

            ui.label(format!("Hello from {}", self.name));
            show_color(ui, self.color, Vec2::new(120.0, 120.0));
            show_color(ui, self.color, Vec2::new(120.0, 120.0));
            show_color(ui, self.color, Vec2::new(120.0, 120.0));
            show_color(ui, self.color, Vec2::new(120.0, 120.0));
            show_color(ui, self.color, Vec2::new(120.0, 120.0));
        });
    }
}

// from egui example
/// Show a color with background checkers to demonstrate transparency (if any).
pub fn show_color(ui: &mut Ui, color: impl Into<Color32>, desired_size: Vec2)
        -> Response {
    show_color32(ui, color.into(), desired_size)
}

fn show_color32(ui: &mut Ui, color: Color32, desired_size: Vec2) -> Response {
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    if ui.is_rect_visible(rect) {
        show_color_at(ui.painter(), color, rect);
    }
    response
}

/// Show a color with background checkers to demonstrate transparency (if any).
pub fn show_color_at(painter: &Painter, color: Color32, rect: Rect) {
    if color.is_opaque() {
        painter.rect_filled(rect, 0.0, color);
    } else {
        // Transparent: how both the transparent and opaque versions of the color
        // background_checkers(painter, rect);

        if color == Color32::TRANSPARENT {
            // There is no opaque version, so just show the background checkers
        } else {
            let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
            let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
            painter.rect_filled(left, 0.0, color);
            painter.rect_filled(right, 0.0, color.to_opaque());
        }
    }
}

