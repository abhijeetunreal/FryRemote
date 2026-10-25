//! The egui user interface: TV list, pairing, and the remote itself.

use crate::controller::{Controller, Notify};
use crate::keys;
use crate::state::*;
use eframe::egui::{self, pos2, vec2, Color32, RichText, Sense, Shape, Stroke, Ui, Vec2};
use std::path::PathBuf;
use std::sync::Arc;

const BG: Color32 = Color32::from_rgb(3, 9, 19);
const PANEL: Color32 = Color32::from_rgb(10, 20, 35);
const PANEL_ALT: Color32 = Color32::from_rgb(15, 27, 44);
const BTN: Color32 = Color32::from_rgb(18, 31, 52);
const BTN_DOWN: Color32 = Color32::from_rgb(35, 56, 92);
const ACCENT: Color32 = Color32::from_rgb(35, 165, 255);
const ACCENT_SOFT: Color32 = Color32::from_rgb(20, 117, 188);
const GOOD: Color32 = Color32::from_rgb(80, 220, 140);
const WARN: Color32 = Color32::from_rgb(244, 186, 77);
const BAD: Color32 = Color32::from_rgb(255, 95, 95);
const TEXT: Color32 = Color32::from_rgb(235, 244, 255);
const MUTED: Color32 = Color32::from_rgb(123, 144, 175);

#[derive(Clone)]
enum Screen {
    List,
    Pair(Tv),
    Remote,
}

pub struct RemoteApp {
    ctrl: Controller,
    screen: Screen,
    ip_input: String,
    code_input: String,
    show_ip_pad: bool,
    show_kbd: bool,
    show_touch: bool,
    touch_acc: Vec2,
    auto_scanned: bool,
}

impl RemoteApp {
    pub fn new(cc: &eframe::CreationContext<'_>, data_dir: PathBuf) -> Self {
        let ctx = cc.egui_ctx.clone();
        let notify: Notify = Arc::new(move || ctx.request_repaint());
        setup_style(&cc.egui_ctx);
        Self {
            ctrl: Controller::new(data_dir, notify),
            screen: Screen::List,
            ip_input: String::new(),
            code_input: String::new(),
            show_ip_pad: false,
            show_kbd: false,
            show_touch: false,
            touch_acc: Vec2::ZERO,
            auto_scanned: false,
        }
    }
}

fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.window_fill = BG;
    style.visuals.panel_fill = BG;
    style.visuals.widgets.inactive.bg_fill = BTN;
    style.visuals.widgets.inactive.weak_bg_fill = BTN;
    style.visuals.widgets.hovered.bg_fill = BTN_DOWN;
    style.visuals.widgets.hovered.weak_bg_fill = BTN_DOWN;
    style.visuals.widgets.active.bg_fill = BTN_DOWN;
    style.visuals.widgets.active.weak_bg_fill = BTN_DOWN;
    style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    style.spacing.item_spacing = vec2(10.0, 10.0);
    style.spacing.button_padding = vec2(14.0, 12.0);
    for (_, f) in style.text_styles.iter_mut() {
        f.size = (f.size * 1.18).max(16.0);
    }
    ctx.set_global_style(style);
}

impl eframe::App for RemoteApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ctrl.poll();
        let snap = self.ctrl.snapshot();
        let mut next: Option<Screen> = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::symmetric(14, 18)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    match self.screen.clone() {
                        Screen::List => self.list_screen(ui, &snap, &mut next),
                        Screen::Pair(tv) => self.pair_screen(ui, &snap, &tv, &mut next),
                        Screen::Remote => self.remote_screen(ui, &snap, &mut next),
                    }
                });
            });

        if let Some(s) = next {
            self.screen = s;
        }
    }
}

// ---------------------------------------------------------------------------
// TV list
// ---------------------------------------------------------------------------

impl RemoteApp {
    fn list_screen(&mut self, ui: &mut Ui, snap: &Shared, next: &mut Option<Screen>) {
        if !self.auto_scanned && self.ctrl.creds_ready() {
            self.auto_scanned = true;
            self.ctrl.scan();
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let icon = egui::Rect::from_min_size(ui.cursor().min, vec2(28.0, 28.0));
            ui.painter().circle_filled(icon.center(), 14.0, ACCENT);
            ui.painter().circle_filled(icon.center() + vec2(0.0, 0.0), 8.0, BG);
            ui.painter().rect_filled(
                egui::Rect::from_center_size(icon.center(), vec2(7.0, 7.0)),
                3.0,
                BG,
            );
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.label(RichText::new("FryRemote").size(24.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::CircleShape::filled(ACCENT, 7.0, pos2(0.0, 0.0)));
                    ui.label(RichText::new("Connected").color(MUTED).small());
                });
            });
        });
        ui.label(RichText::new("Your Google TV Remote").color(MUTED).small());
        ui.add_space(14.0);

        if let Some(e) = self.ctrl.creds_error() {
            ui.colored_label(BAD, format!("Could not create secure identity: {e}"));
        } else if !self.ctrl.creds_ready() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Preparing secure identity (first launch only)...");
            });
        }

        // Saved TVs
        let saved = self.ctrl.store.tvs.clone();
        if !saved.is_empty() {
            ui.label(RichText::new("Your TVs").strong());
        }
        for tv in &saved {
            egui::Frame::new()
                .fill(BTN)
                .corner_radius(14)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(&tv.name).size(20.0).strong());
                    ui.label(
                        RichText::new(format!(
                            "{}  -  {}",
                            tv.host,
                            if tv.paired { "paired" } else { "not paired yet" }
                        ))
                        .color(if tv.paired { GOOD } else { WARN }),
                    );
                    ui.horizontal(|ui| {
                        let open = if tv.paired { "Open remote" } else { "Pair" };
                        if ui.add_enabled(self.ctrl.creds_ready(), egui::Button::new(open).fill(ACCENT)).clicked() {
                            if tv.paired {
                                self.ctrl.connect(tv);
                                *next = Some(Screen::Remote);
                            } else {
                                self.code_input.clear();
                                self.ctrl.start_pairing(tv);
                                *next = Some(Screen::Pair(tv.clone()));
                            }
                        }
                        if tv.paired && ui.button("Re-pair").clicked() {
                            self.code_input.clear();
                            self.ctrl.start_pairing(tv);
                            *next = Some(Screen::Pair(tv.clone()));
                        }
                        if ui.button("Forget").clicked() {
                            self.ctrl.store.remove(&tv.host);
                        }
                    });
                });
        }

        // Discovery
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("On your network").strong());
            if snap.scanning {
                ui.spinner();
            } else if ui.button("Scan").clicked() {
                self.ctrl.scan();
            }
        });
        let new_found: Vec<Tv> = snap
            .discovered
            .iter()
            .filter(|d| !saved.iter().any(|s| s.host == d.host))
            .cloned()
            .collect();
        if new_found.is_empty() && !snap.scanning {
            ui.label(RichText::new("No new TVs found. Make sure the TV is on and on the same Wi-Fi, or add it by IP below.").color(Color32::GRAY));
        }
        for tv in new_found {
            ui.horizontal(|ui| {
                ui.label(format!("{}  ({})", tv.name, tv.host));
                if ui.button("Add").clicked() {
                    self.ctrl.store.upsert(tv.clone());
                }
            });
        }

        // Manual IP
        ui.add_space(8.0);
        if ui.button(if self.show_ip_pad { "Hide manual entry" } else { "Add TV by IP address" }).clicked() {
            self.show_ip_pad = !self.show_ip_pad;
        }
        if self.show_ip_pad {
            ui.label("TV IP address (Settings > Network on the TV):");
            ui.label(RichText::new(if self.ip_input.is_empty() { "_" } else { &self.ip_input }).size(26.0).monospace());
            keypad(ui, &mut self.ip_input, false, 15);
            let valid = self.ip_input.parse::<std::net::Ipv4Addr>().is_ok();
            if ui.add_enabled(valid, egui::Button::new("Save TV").fill(ACCENT)).clicked() {
                let host = self.ip_input.clone();
                self.ctrl.store.upsert(Tv { name: format!("TV {host}"), host, paired: false });
                self.ip_input.clear();
                self.show_ip_pad = false;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Pairing
    // -----------------------------------------------------------------------

    fn pair_screen(&mut self, ui: &mut Ui, snap: &Shared, tv: &Tv, next: &mut Option<Screen>) {
        ui.heading(format!("Pair with {}", tv.name));
        ui.add_space(6.0);

        match &snap.pairing {
            PairingUi::None | PairingUi::Starting => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Connecting to the TV...");
                });
            }
            PairingUi::WaitingForCode | PairingUi::WrongCode | PairingUi::Verifying => {
                ui.label("A 6-character code is showing on your TV. Enter it here:");
                if snap.pairing == PairingUi::WrongCode {
                    ui.colored_label(BAD, "That code doesn't match. Check the TV and try again.");
                }
                let shown = format!("{:_<6}", self.code_input.to_uppercase());
                ui.label(RichText::new(shown).size(36.0).monospace().color(ACCENT));
                keypad(ui, &mut self.code_input, true, 6);
                let verifying = snap.pairing == PairingUi::Verifying;
                if verifying {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Verifying...");
                    });
                }
                let ready = self.code_input.len() == 6 && !verifying;
                if ui.add_enabled(ready, egui::Button::new("Pair").fill(ACCENT)).clicked() {
                    self.ctrl.submit_code(&self.code_input);
                }
            }
            PairingUi::Done => {
                ui.colored_label(GOOD, "Paired!");
                let mut tv = tv.clone();
                tv.paired = true;
                self.ctrl.store.mark_paired(&tv.host);
                self.ctrl.connect(&tv);
                *next = Some(Screen::Remote);
            }
            PairingUi::Error(e) => {
                ui.colored_label(BAD, format!("Pairing failed: {e}"));
                if ui.button("Try again").clicked() {
                    self.code_input.clear();
                    self.ctrl.start_pairing(tv);
                }
            }
        }

        ui.add_space(10.0);
        if ui.button("Cancel").clicked() {
            self.ctrl.cancel_pairing();
            *next = Some(Screen::List);
        }
    }

    // -----------------------------------------------------------------------
    // The remote
    // -----------------------------------------------------------------------

    fn remote_screen(&mut self, ui: &mut Ui, snap: &Shared, next: &mut Option<Screen>) {
        let tv = self.ctrl.active.clone();

        // Header
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("<").clicked() {
                self.ctrl.disconnect();
                *next = Some(Screen::List);
            }
            ui.add_space(8.0);
            if let Some(tv) = &tv {
                ui.label(RichText::new(&tv.name).size(22.0).strong());
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (color, text) = match &snap.status {
                    ConnStatus::Connected => (GOOD, "Connected"),
                    ConnStatus::Connecting => (WARN, "Connecting"),
                    ConnStatus::Failed(_) => (BAD, "Offline"),
                    ConnStatus::Idle => (MUTED, "Idle"),
                };
                ui.horizontal(|ui| {
                    ui.add(egui::CircleShape::filled(color, 6.0, pos2(0.0, 0.0)));
                    ui.label(RichText::new(text).color(color).small());
                });
            });
        });

        if let ConnStatus::Failed(msg) = &snap.status {
            ui.colored_label(BAD, msg);
            ui.horizontal(|ui| {
                if ui.button("Retry").clicked() {
                    if let Some(tv) = &tv {
                        self.ctrl.connect(tv);
                    }
                }
                if ui.button("Pair again").clicked() {
                    if let Some(tv) = tv.clone() {
                        self.ctrl.disconnect();
                        self.code_input.clear();
                        self.ctrl.start_pairing(&tv);
                        *next = Some(Screen::Pair(tv));
                    }
                }
            });
        }

        let w = ui.available_width();
        let gap = ui.spacing().item_spacing.x;

        // Power + status
        ui.horizontal(|ui| {
            if press_button(ui, "Power", vec2(100.0, 48.0), BAD) {
                self.ctrl.key(keys::POWER);
            }
            if let Some((lvl, max, muted)) = snap.volume {
                let frac = if max > 0 { lvl as f32 / max as f32 } else { 0.0 };
                ui.add(
                    egui::ProgressBar::new(frac)
                        .desired_width(ui.available_width())
                        .text(if muted { "Muted".to_string() } else { format!("Volume {lvl}/{max}") }),
                );
            }
        });
        if let Some(app) = &snap.current_app {
            ui.label(RichText::new(format!("Now: {app}")).color(Color32::GRAY).small());
        }

        // D-pad
        ui.add_space(18.0);
        let cell = ((w - 2.0 * gap) / 3.0).min(125.0);
        let sz = vec2(cell, cell * 0.90);
        centered(ui, 3.0 * cell + 2.0 * gap, |ui| {
            ui.add_space(cell + gap);
            if arrow_button(ui, Dir::Up, sz) {
                self.ctrl.key(keys::DPAD_UP);
            }
        });
        centered(ui, 3.0 * cell + 2.0 * gap, |ui| {
            if arrow_button(ui, Dir::Left, sz) {
                self.ctrl.key(keys::DPAD_LEFT);
            }
            if press_button(ui, "OK", sz, ACCENT) {
                self.ctrl.key(keys::DPAD_CENTER);
            }
            if arrow_button(ui, Dir::Right, sz) {
                self.ctrl.key(keys::DPAD_RIGHT);
            }
        });
        centered(ui, 3.0 * cell + 2.0 * gap, |ui| {
            ui.add_space(cell + gap);
            if arrow_button(ui, Dir::Down, sz) {
                self.ctrl.key(keys::DPAD_DOWN);
            }
        });

        // Back / Home / Menu
        if let Some(k) = button_row(ui, &[("Back", keys::BACK), ("Home", keys::HOME), ("Menu", keys::MENU)], 52.0) {
            self.ctrl.key(k);
        }

        // Volume / channel
        ui.add_space(4.0);
        let vsz = vec2((w - 3.0 * gap) / 4.0, 56.0);
        ui.horizontal(|ui| {
            if hold_button(ui, "Vol -", vsz) {
                self.ctrl.key(keys::VOLUME_DOWN);
            }
            if press_button(ui, "Mute", vsz, BTN) {
                self.ctrl.key(keys::VOLUME_MUTE);
            }
            if hold_button(ui, "Vol +", vsz) {
                self.ctrl.key(keys::VOLUME_UP);
            }
            if press_button(ui, "Input", vsz, BTN) {
                self.ctrl.key(keys::TV_INPUT);
            }
        });
        ui.horizontal(|ui| {
            if press_button(ui, "Ch -", vsz, BTN) {
                self.ctrl.key(keys::CHANNEL_DOWN);
            }
            if press_button(ui, "Ch +", vsz, BTN) {
                self.ctrl.key(keys::CHANNEL_UP);
            }
            if press_button(ui, "Guide", vsz, BTN) {
                self.ctrl.key(keys::GUIDE);
            }
            if press_button(ui, "Info", vsz, BTN) {
                self.ctrl.key(keys::INFO);
            }
        });

        // Media
        if let Some(k) = button_row(
            ui,
            &[("<<", keys::MEDIA_REWIND), ("Play/Pause", keys::MEDIA_PLAY_PAUSE), (">>", keys::MEDIA_FAST_FORWARD)],
            52.0,
        ) {
            self.ctrl.key(k);
        }
        if let Some(k) = button_row(
            ui,
            &[("Prev", keys::MEDIA_PREVIOUS), ("Stop", keys::MEDIA_STOP), ("Next", keys::MEDIA_NEXT)],
            48.0,
        ) {
            self.ctrl.key(k);
        }
        if let Some(k) = button_row(
            ui,
            &[("Search", keys::SEARCH), ("Captions", keys::CAPTIONS), ("Settings", keys::SETTINGS)],
            48.0,
        ) {
            self.ctrl.key(k);
        }

        // Apps
        ui.add_space(6.0);
        ui.label(RichText::new("Apps").strong());
        let asz = vec2((w - 2.0 * gap) / 3.0, 48.0);
        for chunk in keys::APPS.chunks(3) {
            ui.horizontal(|ui| {
                for (name, link) in chunk {
                    if press_button(ui, name, asz, BTN) {
                        self.ctrl.launch(link);
                    }
                }
            });
        }

        // Touchpad
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_touch, "Touchpad");
            ui.checkbox(&mut self.show_kbd, "Keyboard");
        });
        if self.show_touch {
            self.touchpad(ui, w);
        }
        if self.show_kbd {
            self.keyboard(ui, w);
        }
        ui.add_space(20.0);
    }

    fn touchpad(&mut self, ui: &mut Ui, w: f32) {
        let (rect, resp) = ui.allocate_exact_size(vec2(w, 190.0), Sense::click_and_drag());
        ui.painter().rect_filled(rect, 18.0, BTN);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Swipe to move - tap to select",
            egui::FontId::proportional(16.0),
            Color32::GRAY,
        );
        const STEP: f32 = 46.0;
        if resp.dragged() {
            self.touch_acc += resp.drag_delta();
            while self.touch_acc.x > STEP {
                self.ctrl.key(keys::DPAD_RIGHT);
                self.touch_acc.x -= STEP;
            }
            while self.touch_acc.x < -STEP {
                self.ctrl.key(keys::DPAD_LEFT);
                self.touch_acc.x += STEP;
            }
            while self.touch_acc.y > STEP {
                self.ctrl.key(keys::DPAD_DOWN);
                self.touch_acc.y -= STEP;
            }
            while self.touch_acc.y < -STEP {
                self.ctrl.key(keys::DPAD_UP);
                self.touch_acc.y += STEP;
            }
        } else {
            self.touch_acc = Vec2::ZERO;
        }
        if resp.clicked() {
            self.ctrl.key(keys::DPAD_CENTER);
        }
    }

    fn keyboard(&mut self, ui: &mut Ui, w: f32) {
        ui.label(RichText::new("Type on the TV").strong());
        ui.scope(|ui| {
            // Tight padding so ten keys always fit across a narrow phone screen.
            let gap = 5.0;
            ui.spacing_mut().item_spacing.x = gap;
            ui.spacing_mut().button_padding = vec2(1.0, 8.0);
            let kw = ((w - 9.0 * gap) / 10.0).floor();
            for row in ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"] {
                let width = row.len() as f32 * kw + (row.len() as f32 - 1.0) * gap;
                centered(ui, width, |ui| {
                    for c in row.chars() {
                        if press_button(ui, &c.to_string(), vec2(kw, 48.0), BTN) {
                            if let Some(k) = keys::key_for_char(c) {
                                self.ctrl.key(k);
                            }
                        }
                    }
                });
            }
            ui.horizontal(|ui| {
                let small = vec2(kw * 1.4, 48.0);
                for (label, c) in [(".", '.'), ("@", '@'), ("-", '-'), ("/", '/')] {
                    if press_button(ui, label, small, BTN) {
                        if let Some(k) = keys::key_for_char(c) {
                            self.ctrl.key(k);
                        }
                    }
                }
            });
            ui.horizontal(|ui| {
                if press_button(ui, "Space", vec2(w * 0.40, 52.0), BTN) {
                    self.ctrl.key(keys::SPACE);
                }
                if hold_button(ui, "Del", vec2(w * 0.25, 52.0)) {
                    self.ctrl.key(keys::DEL);
                }
                if press_button(ui, "Enter", vec2(w * 0.28, 52.0), ACCENT) {
                    self.ctrl.key(keys::ENTER);
                }
            });
        });
    }
}

// ---------------------------------------------------------------------------
// Widgets
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// Run `f` in a horizontal row centred in the available width.
fn centered(ui: &mut Ui, content_width: f32, f: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        let pad = ((ui.available_width() - content_width) / 2.0).max(0.0);
        ui.add_space(pad);
        f(ui);
    });
}

/// A plain button that fires once on click.
fn press_button(ui: &mut Ui, label: &str, size: Vec2, fill: Color32) -> bool {
    ui.add_sized(size, egui::Button::new(RichText::new(label).size(18.0)).fill(fill).corner_radius(12))
        .clicked()
}

/// Evenly spaced row of buttons; returns the key code of whichever was tapped.
fn button_row(ui: &mut Ui, items: &[(&str, i32)], height: f32) -> Option<i32> {
    let gap = ui.spacing().item_spacing.x;
    let w = (ui.available_width() - gap * (items.len() as f32 - 1.0)) / items.len() as f32;
    let mut hit = None;
    ui.horizontal(|ui| {
        for (label, code) in items {
            if press_button(ui, label, vec2(w, height), BTN) {
                hit = Some(*code);
            }
        }
    });
    hit
}

/// Fires on press, then repeats while held (volume, deletes).
fn hold_button(ui: &mut Ui, label: &str, size: Vec2) -> bool {
    let resp = ui.add_sized(size, egui::Button::new(RichText::new(label).size(18.0)).fill(BTN).corner_radius(12));
    hold_logic(ui, &resp)
}

fn hold_logic(ui: &Ui, resp: &egui::Response) -> bool {
    let id = resp.id.with("hold");
    let now = ui.input(|i| i.time);
    if resp.is_pointer_button_down_on() {
        let next: Option<f64> = ui.data(|d| d.get_temp(id));
        ui.ctx().request_repaint();
        match next {
            None => {
                ui.data_mut(|d| d.insert_temp(id, now + 0.45));
                true
            }
            Some(t) if now >= t => {
                ui.data_mut(|d| d.insert_temp(id, now + 0.12));
                true
            }
            _ => false,
        }
    } else {
        ui.data_mut(|d| d.remove::<f64>(id));
        false
    }
}

/// D-pad arrow drawn with a painter so it doesn't depend on font glyphs.
fn arrow_button(ui: &mut Ui, dir: Dir, size: Vec2) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fire = hold_logic(ui, &resp);
    let fill = if resp.is_pointer_button_down_on() { BTN_DOWN } else { BTN };
    ui.painter().rect_filled(rect, 14.0, fill);
    let c = rect.center();
    let s = size.min_elem() * 0.2;
    let pts = match dir {
        Dir::Up => vec![c + vec2(0.0, -s), c + vec2(-s * 1.2, s * 0.8), c + vec2(s * 1.2, s * 0.8)],
        Dir::Down => vec![c + vec2(0.0, s), c + vec2(-s * 1.2, -s * 0.8), c + vec2(s * 1.2, -s * 0.8)],
        Dir::Left => vec![c + vec2(-s, 0.0), c + vec2(s * 0.8, -s * 1.2), c + vec2(s * 0.8, s * 1.2)],
        Dir::Right => vec![c + vec2(s, 0.0), c + vec2(-s * 0.8, -s * 1.2), c + vec2(-s * 0.8, s * 1.2)],
    };
    ui.painter().add(Shape::convex_polygon(pts, Color32::WHITE, Stroke::NONE));
    let _ = pos2(0.0, 0.0);
    fire
}

/// On-screen keypad so entering codes and IPs never depends on the system keyboard.
fn keypad(ui: &mut Ui, text: &mut String, hex: bool, max_len: usize) {
    let rows: &[&str] = if hex {
        &["0123", "4567", "89AB", "CDEF"]
    } else {
        &["123", "456", "789", ".0"]
    };
    let gap = ui.spacing().item_spacing.x;
    let cols = if hex { 4.0 } else { 3.0 };
    let w = ((ui.available_width() - gap * (cols - 1.0)) / cols).min(90.0);
    for row in rows {
        ui.horizontal(|ui| {
            for c in row.chars() {
                if press_button(ui, &c.to_string(), vec2(w, 52.0), BTN) && text.len() < max_len {
                    text.push(c);
                }
            }
            if !hex && *row == ".0" && press_button(ui, "Del", vec2(w, 52.0), BTN) {
                text.pop();
            }
        });
    }
    if hex {
        ui.horizontal(|ui| {
            if press_button(ui, "Del", vec2(w * 2.0 + gap, 48.0), BTN) {
                text.pop();
            }
            if press_button(ui, "Clear", vec2(w * 2.0 + gap, 48.0), BTN) {
                text.clear();
            }
        });
    }
}
