#![feature(arc_unwrap_or_clone)]
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::fmt;

use eframe::egui;
use egui::color::{Color32};
use egui::{Sense, Response, Vec2, Painter, Ui, Rect};

use leds;
use leds::msg_handler::MsgHandler;
use leds::chan_description::{ChanDescription, HasChanDescriptions};
use leds::chan::ChanConfig;
use proto::v1::ChanId;
use proto::v1::Msg;

use leds::udp_srv::UdpSrv;

struct LedReceiver {
    chans: Vec<Color32>,
    ctx: egui::Context,
}
impl LedReceiver {
    fn new(num_chans: usize, ctx: egui::Context) -> Self {
        LedReceiver {
            chans: vec![Color32::from_rgb(0xAA, 0xAA, 0xAA); num_chans],
            ctx,
        }
    }
}

impl fmt::Debug for LedReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LedReceiver {:?}", self.chans)
    }
}

impl fmt::Display for LedReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LedReceiver (TODO Display)")
    }
}

impl HasChanDescriptions for LedReceiver {
    fn chans(&self) -> Vec<(ChanId, String)> {
        let mut ret = Vec::new();

        for (ii, _rgb_chan) in self.chans.iter().enumerate() {
            ret.push((ChanId( (ii * 3 + 0) as u16 ), format!("RGB {} R", ii)));
            ret.push((ChanId( (ii * 3 + 1) as u16 ), format!("RGB {} G", ii)));
            ret.push((ChanId( (ii * 3 + 2) as u16 ), format!("RGB {} B", ii)));
        }

        ret
    }

    fn chan_descriptions(&self) -> Vec<ChanDescription> {
        let mut ret = Vec::new();

        for (ii, _rgb_chan) in self.chans.iter().enumerate() {
            ret.push(ChanDescription::new(
                    (ii * 3 + 0) as u16,
                    format!("RGB chan {} R", ii),
                    ChanConfig::default()));
            ret.push(ChanDescription::new(
                    (ii * 3 + 1) as u16,
                    format!("RGB chan {} G", ii),
                    ChanConfig::default()));
            ret.push(ChanDescription::new(
                    (ii * 3 + 2) as u16,
                    format!("RGB chan {} B", ii),
                    ChanConfig::default()));
        }

        ret
    }
}

use proto::v1::{Val, ChanVal};

impl MsgHandler for LedReceiver {
    fn handle_msg(&mut self, msg: &Msg) -> Result<(), String> {
        for ChanVal(ChanId(cid), val) in msg.vals.iter() {
            match val {
                Val::F32(fval) => {
                    let rgb_chan = (cid / 3) as usize;
                    let rgb_subchan = cid % 3;

                    if rgb_chan > self.chans.len() {
                        return Err(format!("Invalid RGB channel {rgb_chan}"));
                    }

                    let u8val = (fval * 255.0) as u8;
                    let prev_col = self.chans[rgb_chan].clone();

                    match rgb_subchan {
                        0 => self.chans[rgb_chan] =
                            Color32::from_rgb(u8val, prev_col.g(), prev_col.b()),
                        1 => self.chans[rgb_chan] =
                            Color32::from_rgb(prev_col.r(), u8val, prev_col.b()),
                        2 => self.chans[rgb_chan] =
                            Color32::from_rgb(prev_col.r(), prev_col.g(), u8val),
                        _ => unreachable!()
                    }
                    self.ctx.request_repaint();
                },
                _ => todo!(),
            }
        }
        Ok(())
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            let app = GuiApp::new("leds".to_string(), cc.egui_ctx.clone());
            Box::new(app)
        }),
    );
}

struct GuiApp {
    name: String,
    ctx: egui::Context,
    leds: Arc<Mutex<LedReceiver>>,
}

use std::net::IpAddr;
use std::thread;

impl GuiApp {
    pub fn new(name: String, ctx: egui::Context) -> GuiApp {
        let mut visuals = egui::Visuals::dark();
        visuals.faint_bg_color = Color32::from_rgb(0,0,0);
        visuals.extreme_bg_color = Color32::from_rgb(0,0,0);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0,0,0);
        let mut style = egui::Style::default();
        style.visuals = visuals;
        ctx.set_style(style);

        let leds = Arc::new(Mutex::new(LedReceiver::new(2, ctx.clone())));

        let mut udp = UdpSrv::new(
            Some("127.0.0.1".parse().unwrap()),
            Some(12344),
            leds.clone()).unwrap();


        thread::spawn(move || {
            udp.run();
        });

        GuiApp {
            name,
            ctx,
            leds,
        }
    }
}

impl eframe::App for GuiApp {
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

            // show_color(ui, self.color, Vec2::new(120.0, 120.0));
            // show_color(ui, self.color, Vec2::new(120.0, 120.0));
            // show_color(ui, self.color, Vec2::new(120.0, 120.0));
            // show_color(ui, self.color, Vec2::new(120.0, 120.0));
            // show_color(ui, self.color, Vec2::new(120.0, 120.0));


            let leds = {
                let leds = self.leds.clone();
                let leds = leds.lock().unwrap();
                for color in leds.chans.iter() {
                    show_color(ui, color.clone(), Vec2::new(120.0, 120.0));
                }
            };
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

