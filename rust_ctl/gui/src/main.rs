#![feature(arc_unwrap_or_clone, test)]
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

#[derive(Clone, Copy, Debug)]
enum ChanValue {
    BlackAndWhite(u8),
    Rgb(Color32),
}

impl ChanValue {
    #[allow(dead_code)]
    fn bw(val: u8) -> ChanValue {
        ChanValue::BlackAndWhite(val)
    }

    #[allow(dead_code)]
    fn rgb(r: u8, g: u8, b: u8) -> ChanValue {
        ChanValue::Rgb(Color32::from_rgb(r, g, b))
    }

    fn num_subchans(&self) -> u16 {
        match self {
            ChanValue::BlackAndWhite(_) => 1,
            ChanValue::Rgb(_) => 3,
        }
    }

    fn set_subchan(&mut self, subchan: u8, val: u8) {
        match self {
            ChanValue::BlackAndWhite(current_val) => {
                if subchan > 0 {
                    panic!("Invalid subchan value {subchan} for BW channel")
                }
                *current_val = val
            },
            ChanValue::Rgb(col) => {
                match subchan {
                    0 => *col = Color32::from_rgb(val,     col.g(), col.b()),
                    1 => *col = Color32::from_rgb(col.r(), val,     col.b()),
                    2 => *col = Color32::from_rgb(col.r(), col.g(), val),
                    _ => {
                        panic!(
                            "invalid subchan value {subchan} for RGB channel")
                    }
                }
            }
        }
    }
}

impl Into<Color32> for ChanValue {
    fn into(self) -> Color32 {
        match self {
            ChanValue::Rgb(col) => col,
            ChanValue::BlackAndWhite(val)
                => Color32::from_rgb(val, val, val),
        }
    }
}

#[derive(Debug, Clone)]
struct Channel {
    name: String,
    value: ChanValue,
}

impl Into<Color32> for &Channel {
    fn into(self) -> Color32 {
        self.value.into()
    }
}

impl Channel {
    fn rgb(name: String, r: u8, g: u8, b: u8) -> Self {
        Channel {
            name,
            value: ChanValue::Rgb(Color32::from_rgb(r, g, b)),
        }
    }

    fn bw(name: String, value: u8) -> Self {
        Channel {
            name,
            value: ChanValue::BlackAndWhite(value),
        }
    }

    fn num_subchans(&self) -> u16 {
        self.value.num_subchans()
    }

    fn set_subchan(&mut self, subchan: u8, val: u8) {
        self.value.set_subchan(subchan, val)
    }
}

struct LedReceiver {
    chans: Vec<Channel>,
    update_callback: Option<Box<dyn Fn() -> () + Send + Sync>>,
}
impl LedReceiver {
    fn new(
        chans: Vec<Channel>,
        update_callback: Option<Box<dyn Fn() -> () + Send + Sync>>
    ) -> Self {
        LedReceiver { chans, update_callback }
    }

    fn get_cid_index(&self, cid: u16) -> Option<(u16, u8)> {
        let mut index: u16 = 0;

        for (ii, ch) in self.chans.iter().enumerate() {
            let subchans = ch.num_subchans();

            if cid >= index && cid < (index + subchans) {
                return Some((ii as u16, (cid - index) as u8))
            }

            index += 1;
        }

        None
    }

    fn set_chan_val(&mut self, cid: u16, val: u8) -> Result<(), ()> {
        let (index, subchan_id) = self.get_cid_index(cid).ok_or(())?;

        self.chans[index as usize].set_subchan(subchan_id, val);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;

    #[test]
    fn test_get_cid_index() {
        let mut chans = vec![Channel::bw("blah".to_string(), 0)];
        let led_recv = LedReceiver::new(chans.clone(), None);
        assert_eq!(Some((0, 0)), led_recv.get_cid_index(0));
        assert_eq!(None,         led_recv.get_cid_index(1));

        chans.push(Channel::rgb("blah".to_string(), 0,0,0));
        let led_recv = LedReceiver::new(chans.clone(), None);

        assert_eq!(Some((0, 0)), led_recv.get_cid_index(0));
        assert_eq!(Some((1, 0)), led_recv.get_cid_index(1));
        assert_eq!(Some((1, 1)), led_recv.get_cid_index(2));
        assert_eq!(Some((1, 2)), led_recv.get_cid_index(3));
        assert_eq!(None,         led_recv.get_cid_index(4));
    }

    #[test]
    fn test_colors() {
        let mut bw = ChanValue::bw(10);
        assert_eq!(Color32::from_rgb(10,10,10), bw.into());
        bw.set_subchan(0, 20);
        assert_eq!(Color32::from_rgb(20,20,20,), bw.into());
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

                    if rgb_chan > self.chans.len() {
                        return Err(format!("Invalid RGB channel {rgb_chan}"));
                    }

                    let u8val = (fval * 255.0) as u8;
                    if let Err(()) = self.set_chan_val(*cid, u8val) {
                        return Err(format!("Invalid cid {cid}"));
                    }

                    if let Some(cb) = &self.update_callback {
                        (cb)();
                    }
                },
                _ => todo!(),
            }
        }
        Ok(())
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let channels = vec![
        Channel::bw( "1".to_string(), 100),
        Channel::rgb("2".to_string(), 0xff, 0x50, 0x10),
    ];
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            let app = GuiApp::new(
                "leds".to_string(), channels, cc.egui_ctx.clone());
            Box::new(app)
        }),
    );
}

struct GuiApp {
    name: String,
    leds: Arc<Mutex<LedReceiver>>,
}

use std::thread;

impl GuiApp {
    pub fn new(name: String, chans: Vec<Channel>, ctx: egui::Context) -> GuiApp {
        let mut visuals = egui::Visuals::dark();
        visuals.faint_bg_color = Color32::from_rgb(0,0,0);
        visuals.extreme_bg_color = Color32::from_rgb(0,0,0);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0,0,0);
        let mut style = egui::Style::default();
        style.visuals = visuals;
        ctx.set_style(style);

        let lctx = ctx.clone();
        let leds = Arc::new(Mutex::new(
                LedReceiver::new(
                    chans, Some(Box::new(move || lctx.request_repaint())))));

        let mut udp = UdpSrv::new(
            Some("127.0.0.1".parse().unwrap()),
            Some(12344),
            leds.clone()).unwrap();


        thread::spawn(move || {
            udp.run();
        });

        GuiApp {
            name,
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

            let leds = self.leds.clone();
            let leds = leds.lock().unwrap();
            for chan in leds.chans.iter() {
                egui::Window::new(&chan.name).show(ctx, |ui| {
                    show_color(ui, chan, Vec2::new(120.0, 120.0));
                });
            }
        });
    }
}

// from egui example
/// Show a color with background checkers to demonstrate transparency (if any).
pub fn show_color(ui: &mut Ui, color: impl Into<Color32>, desired_size: Vec2)
        -> Response {
    let col: Color32 = color.into();
    ui.label(format!("{} {} {}", col.r(), col.g(), col.b()));
    show_color32(ui, col, desired_size)
}

fn show_color32(ui: &mut Ui, color: Color32, desired_size: Vec2) -> Response {
    let (rect, response) =
        ui.allocate_at_least(desired_size, Sense::hover());
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
            let left =
                Rect::from_min_max(rect.left_top(), rect.center_bottom());
            let right =
                Rect::from_min_max(rect.center_top(), rect.right_bottom());
            painter.rect_filled(left, 0.0, color);
            painter.rect_filled(right, 0.0, color.to_opaque());
        }
    }
}

