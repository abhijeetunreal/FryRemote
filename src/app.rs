//! The egui user interface for FryRemote: exact 1:1 match with modern Android TV remote design.

use crate::controller::{Controller, Notify};
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::state::*;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Theme Colors (matched to the design mock)
// ---------------------------------------------------------------------------

pub const BG: Color32 = Color32::from_rgb(9, 13, 22); // #090D16 Deep Slate Navy
pub const CARD_BG: Color32 = Color32::from_rgb(15, 23, 42); // #0F172A Mock Dark Card
pub const BTN_BG: Color32 = Color32::from_rgb(17, 28, 46); // #111C2E Button Surface
pub const BTN_HOVER: Color32 = Color32::from_rgb(26, 42, 68); // #1A2A44
pub const BTN_ACTIVE: Color32 = Color32::from_rgb(36, 56, 92); // #24385C
pub const BORDER: Color32 = Color32::from_rgb(28, 43, 68); // #1C2B44
pub const ACCENT: Color32 = Color32::from_rgb(59, 130, 246); // #3B82F6 Primary Electric Blue
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(96, 165, 250); // #60A5FA
pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(29, 78, 216); // #1D4ED8
pub const GREEN_TEXT: Color32 = Color32::from_rgb(34, 197, 94); // #22C55E Connected Green
pub const GREEN_BG: Color32 = Color32::from_rgb(11, 39, 32); // #0B2720
pub const GREEN_BORDER: Color32 = Color32::from_rgb(18, 71, 56); // #124738
pub const RED_POWER: Color32 = Color32::from_rgb(239, 68, 68); // #EF4444 Crimson
pub const TEXT_WHITE: Color32 = Color32::from_rgb(248, 250, 252); // #F8FAFC Light
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139); // #64748B Slate Secondary
pub const ICON_SLATE: Color32 = Color32::from_rgb(148, 163, 184); // #94A3B8

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Remote,
    Apps,
    Touchpad,
    More,
}

#[derive(Clone)]
enum Screen {
    Remote,
    Pair(Tv),
}

pub struct RemoteApp {
    ctrl: Controller,
    screen: Screen,
    tab: Tab,
    show_tv_selector: bool,
    ip_input: String,
    code_input: String,
    show_ip_pad: bool,
    touch_acc: Vec2,
    auto_scanned: bool,
    sim_volume: f32,
}

impl RemoteApp {
    pub fn new(cc: &eframe::CreationContext<'_>, data_dir: PathBuf) -> Self {
        let ctx = cc.egui_ctx.clone();
        let notify: Notify = Arc::new(move || ctx.request_repaint());
        setup_style(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app = Self {
            ctrl: Controller::new(data_dir, notify),
            screen: Screen::Remote,
            tab: Tab::Remote,
            show_tv_selector: false,
            ip_input: String::new(),
            code_input: String::new(),
            show_ip_pad: false,
            touch_acc: Vec2::ZERO,
            auto_scanned: false,
            sim_volume: 0.15,
        };

        // If we have saved TVs, auto-connect to the first paired TV
        if let Some(tv) = app.ctrl.store.tvs.iter().find(|t| t.paired).cloned() {
            app.ctrl.connect(&tv);
        }

        app
    }
}

fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(TEXT_WHITE);
    style.visuals.window_fill = BG;
    style.visuals.panel_fill = BG;
    style.visuals.widgets.inactive.bg_fill = BTN_BG;
    style.visuals.widgets.inactive.weak_bg_fill = BTN_BG;
    style.visuals.widgets.hovered.bg_fill = BTN_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = BTN_HOVER;
    style.visuals.widgets.active.bg_fill = BTN_ACTIVE;
    style.visuals.widgets.active.weak_bg_fill = BTN_ACTIVE;
    style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    style.spacing.item_spacing = vec2(6.0, 6.0);
    style.spacing.button_padding = vec2(8.0, 6.0);
    ctx.set_global_style(style);
}

fn draw_ambient_background(painter: &egui::Painter, rect: Rect) {
    let w = rect.width();
    let wave_fill = Color32::from_rgb(12, 26, 52);
    let wave_crest = Color32::from_rgb(24, 52, 102);

    let n = 32;
    let mut pts = Vec::with_capacity(n + 3);
    let mut crest = Vec::with_capacity(n + 1);
    pts.push(pos2(rect.min.x, rect.max.y));
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let x = rect.min.x + t * w;
        let y = rect.max.y - 72.0 + (t * std::f32::consts::PI * 1.2).sin() * 26.0;
        pts.push(pos2(x, y));
        crest.push(pos2(x, y));
    }
    pts.push(pos2(rect.max.x, rect.max.y));
    painter.add(Shape::convex_polygon(pts, wave_fill, Stroke::NONE));
    painter.add(Shape::line(crest, Stroke::new(1.0, wave_crest)));
}

// ---------------------------------------------------------------------------
// Main App Update
// ---------------------------------------------------------------------------

impl eframe::App for RemoteApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ctrl.poll();
        let snap = self.ctrl.snapshot();
        let mut next: Option<Screen> = None;

        if !self.auto_scanned && self.ctrl.creds_ready() {
            self.auto_scanned = true;
            self.ctrl.scan();
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::symmetric(14, 4)))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                draw_ambient_background(ui.painter(), rect);

                let w = ui.available_width();

                // 1. Top mobile status bar (9:41, Camera Punch Hole, Signal, Wi-Fi, Battery)
                draw_status_bar(ui, w);
                ui.add_space(2.0);

                // 2. Header: Logo, FryRemote title, Settings gear
                self.render_header(ui, &snap, w);
                ui.add_space(8.0);

                // Main content area based on screen and active tab
                match self.screen.clone() {
                    Screen::Pair(tv) => {
                        self.pair_screen(ui, &snap, &tv, &mut next);
                    }
                    Screen::Remote => {
                        // 3. Active TV Selector card ("Living Room / Google TV" + "● Connected" status pill)
                        self.render_tv_card(ui, &snap, w);
                        ui.add_space(8.0);

                        // If TV selector drawer is open
                        if self.show_tv_selector {
                            self.render_tv_selector_drawer(ui, &snap, w, &mut next);
                            ui.add_space(8.0);
                        }

                        let nav_h = 54.0;
                        let scroll_h = (ui.available_height() - nav_h).max(100.0);

                        // Tab Content
                        match self.tab {
                            Tab::Remote => {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                                    .max_height(scroll_h)
                                    .show(ui, |ui| {
                                        self.render_remote_tab(ui, &snap, w, &mut next);
                                    });
                            }
                            Tab::Apps => {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                                    .max_height(scroll_h)
                                    .show(ui, |ui| {
                                        self.render_apps_tab(ui, w);
                                    });
                            }
                            Tab::Touchpad => {
                                self.render_touchpad_tab(ui, w, scroll_h);
                            }
                            Tab::More => {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                                    .max_height(scroll_h)
                                    .show(ui, |ui| {
                                        self.render_more_tab(ui, &snap, w, &mut next);
                                    });
                            }
                        }

                        // 4. Fixed Bottom Navigation Bar (Remote, Apps, Touchpad, More)
                        bottom_nav_bar(ui, &mut self.tab, w);
                    }
                }
            });

        if let Some(s) = next {
            self.screen = s;
        }
    }
}

// ---------------------------------------------------------------------------
// Header Component (Logo, Title, Settings Gear)
// ---------------------------------------------------------------------------

impl RemoteApp {
    fn render_header(&mut self, ui: &mut Ui, _snap: &Shared, _w: f32) {
        ui.horizontal(|ui| {
            // Brand Logo: Vector SVG from assets/icons/brand/fryremote_logo.svg
            let (logo_rect, _) = ui.allocate_exact_size(vec2(28.0, 36.0), Sense::hover());
            svg_brand_logo(ui, logo_rect);

            ui.add_space(8.0);

            // Title: "Fry" (white) + "Remote" (electric blue)
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label(RichText::new("Fry").size(22.0).strong().color(TEXT_WHITE));
                    ui.label(RichText::new("Remote").size(22.0).strong().color(ACCENT));
                });
                ui.label(RichText::new("Your Google TV Remote").size(11.5).color(TEXT_MUTED));
            });

            // Right side: Settings Gear button using vector SVG
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (gear_rect, gear_resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
                let gear_color = if gear_resp.hovered() { TEXT_WHITE } else { ICON_SLATE };
                let icon_size = if gear_resp.is_pointer_button_down_on() { vec2(17.0, 17.0) } else { vec2(19.0, 19.0) };
                let icon_box = Rect::from_center_size(gear_rect.center(), icon_size);
                svg_settings(ui, icon_box, gear_color);
                if gear_resp.clicked() {
                    haptics::trigger(HapticStyle::Light);
                    self.show_tv_selector = !self.show_tv_selector;
                }
            });
        });
    }

    // -----------------------------------------------------------------------
    // TV Selector Card ("Living Room / Google TV" + "● Connected" status pill)
    // -----------------------------------------------------------------------

    fn render_tv_card(&mut self, ui: &mut Ui, snap: &Shared, w: f32) {
        let card_size = vec2(w, 48.0);
        let (rect, resp) = ui.allocate_exact_size(card_size, Sense::click());
        let bg = if resp.is_pointer_button_down_on() {
            BTN_ACTIVE
        } else if resp.hovered() {
            Color32::from_rgb(20, 31, 50)
        } else {
            CARD_BG
        };

        let draw_card_rect = if resp.is_pointer_button_down_on() {
            rect.shrink(0.8)
        } else {
            rect
        };
        let card_stroke = if resp.is_pointer_button_down_on() {
            Stroke::new(1.4, ACCENT)
        } else if resp.hovered() {
            Stroke::new(1.0, Color32::from_rgb(45, 68, 108))
        } else {
            Stroke::new(1.0, BORDER)
        };

        let painter = ui.painter();
        painter.rect_filled(draw_card_rect, 14.0, bg);
        painter.rect_stroke(draw_card_rect, 14.0, card_stroke, StrokeKind::Inside);

        // TV outline icon in electric blue #3B82F6 from vector SVG
        let tv_rect = Rect::from_center_size(pos2(draw_card_rect.min.x + 28.0, draw_card_rect.center().y), vec2(22.0, 16.0));
        svg_tv(ui, tv_rect, ACCENT);

        // Active TV name & subtitle
        let active_name = self
            .ctrl
            .active
            .as_ref()
            .map(|t| t.name.clone())
            .or_else(|| self.ctrl.store.tvs.first().map(|t| t.name.clone()))
            .unwrap_or_else(|| "Living Room".to_string());

        let text_x = draw_card_rect.min.x + 54.0;
        painter.text(
            pos2(text_x, draw_card_rect.center().y - 10.0),
            Align2::LEFT_CENTER,
            &active_name,
            FontId::proportional(15.5),
            TEXT_WHITE,
        );
        painter.text(
            pos2(text_x, draw_card_rect.center().y + 10.0),
            Align2::LEFT_CENTER,
            "Google TV",
            FontId::proportional(12.0),
            TEXT_MUTED,
        );

        // Right side: Connected status pill inside the TV card
        let (pill_color, pill_text, bg_color, border_color) = match &snap.status {
            ConnStatus::Connected => (GREEN_TEXT, "Connected", GREEN_BG, GREEN_BORDER),
            ConnStatus::Connecting => (
                Color32::from_rgb(255, 183, 77),
                "Connecting",
                Color32::from_rgb(38, 28, 12),
                Color32::from_rgb(70, 50, 20),
            ),
            ConnStatus::Failed(_) => (
                Color32::from_rgb(255, 95, 95),
                "Offline",
                Color32::from_rgb(38, 14, 14),
                Color32::from_rgb(70, 25, 25),
            ),
            ConnStatus::Idle => (
                GREEN_TEXT,
                "Connected",
                GREEN_BG,
                GREEN_BORDER,
            ),
        };

        let pill_h = 24.0;
        let pill_w = 90.0;
        let pill_rect = Rect::from_center_size(
            pos2(draw_card_rect.max.x - 14.0 - pill_w * 0.5, draw_card_rect.center().y),
            vec2(pill_w, pill_h),
        );
        painter.rect_filled(pill_rect, 12.0, bg_color);
        painter.rect_stroke(pill_rect, 12.0, Stroke::new(1.0, border_color), StrokeKind::Inside);

        // Green dot + text
        painter.circle_filled(pos2(pill_rect.min.x + 13.0, pill_rect.center().y), 3.5, pill_color);
        painter.text(
            pos2(pill_rect.min.x + 23.0, pill_rect.center().y),
            Align2::LEFT_CENTER,
            pill_text,
            FontId::proportional(11.5),
            pill_color,
        );

        if resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            self.show_tv_selector = !self.show_tv_selector;
        }
    }

    // -----------------------------------------------------------------------
    // TV Selector Drawer / Modal
    // -----------------------------------------------------------------------

    fn render_tv_selector_drawer(&mut self, ui: &mut Ui, snap: &Shared, w: f32, next: &mut Option<Screen>) {
        egui::Frame::new()
            .fill(CARD_BG)
            .corner_radius(16)
            .stroke(Stroke::new(1.0, BORDER))
            .inner_margin(14)
            .show(ui, |ui| {
                ui.set_width(w - 28.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Your Google TVs").size(16.0).strong().color(TEXT_WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕ Close").clicked() {
                            self.show_tv_selector = false;
                        }
                    });
                });
                ui.add_space(8.0);

                // Saved TVs
                for tv in self.ctrl.store.tvs.clone() {
                    ui.horizontal(|ui| {
                        let is_active = self.ctrl.active.as_ref().map(|t| t.host == tv.host).unwrap_or(false);
                        let title = format!("{} ({})", tv.name, tv.host);
                        ui.label(RichText::new(title).color(if is_active { ACCENT } else { TEXT_WHITE }));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Forget").clicked() {
                                self.ctrl.store.remove(&tv.host);
                            }
                            if tv.paired {
                                if !is_active && ui.button("Connect").clicked() {
                                    self.ctrl.connect(&tv);
                                    self.show_tv_selector = false;
                                }
                            } else if ui.button("Pair").clicked() {
                                self.code_input.clear();
                                self.ctrl.start_pairing(&tv);
                                *next = Some(Screen::Pair(tv.clone()));
                                self.show_tv_selector = false;
                            }
                        });
                    });
                }

                // Discovered TVs on Wi-Fi
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Discovered on Wi-Fi").strong().color(TEXT_MUTED));
                    if snap.scanning {
                        ui.spinner();
                    } else if ui.button("Scan").clicked() {
                        self.ctrl.scan();
                    }
                });

                for d in &snap.discovered {
                    if !self.ctrl.store.tvs.iter().any(|t| t.host == d.host) {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} ({})", d.name, d.host));
                            if ui.button("Add & Pair").clicked() {
                                self.ctrl.store.upsert(d.clone());
                                self.ctrl.start_pairing(d);
                                *next = Some(Screen::Pair(d.clone()));
                                self.show_tv_selector = false;
                            }
                        });
                    }
                }

                // Manual IP entry
                ui.add_space(6.0);
                if ui.button(if self.show_ip_pad { "Hide IP Entry" } else { "Add TV by IP Address" }).clicked() {
                    self.show_ip_pad = !self.show_ip_pad;
                }
                if self.show_ip_pad {
                    ui.label(RichText::new(if self.ip_input.is_empty() { "192.168.1._" } else { &self.ip_input }).monospace().size(18.0).color(ACCENT));
                    keypad(ui, &mut self.ip_input, false, 15);
                    let valid = self.ip_input.parse::<std::net::Ipv4Addr>().is_ok();
                    if ui.add_enabled(valid, egui::Button::new("Save & Connect").fill(ACCENT)).clicked() {
                        let host = self.ip_input.clone();
                        let new_tv = Tv { name: format!("Google TV ({host})"), host, paired: false };
                        self.ctrl.store.upsert(new_tv.clone());
                        self.ctrl.start_pairing(&new_tv);
                        *next = Some(Screen::Pair(new_tv));
                        self.ip_input.clear();
                        self.show_ip_pad = false;
                        self.show_tv_selector = false;
                    }
                }
            });
    }

    // -----------------------------------------------------------------------
    // The Remote Tab (Primary Screen matching attached image)
    // -----------------------------------------------------------------------

    fn render_remote_tab(&mut self, ui: &mut Ui, snap: &Shared, w: f32, _next: &mut Option<Screen>) {
        // 1. Power Button & Volume Slider Card Row
        self.render_power_volume_row(ui, snap, w);
        ui.add_space(8.0);

        // 2. D-Pad Directional Arrows & Center OK Button (Glowing Clover Cross)
        self.render_dpad(ui, w);
        ui.add_space(8.0);

        // 3. Navigation Row (Back, Home, Menu)
        self.render_nav_row(ui, w);
        ui.add_space(5.0);

        // 4. Function Grid Row 1 (Vol -, Mute, Vol +, Input)
        self.render_function_row_1(ui, w);
        ui.add_space(5.0);

        // 5. Function Grid Row 2 (Ch -, Ch +, Guide, Info)
        self.render_function_row_2(ui, w);
        ui.add_space(5.0);

        // 6. Media Grid Row 3 (Prev, Play/Pause, Next)
        self.render_function_row_3(ui, w);
        ui.add_space(5.0);

        // 7. Utility Grid Row 4 (Search, Captions, Settings)
        self.render_function_row_4(ui, w);
        ui.add_space(6.0);

        // 8. Apps Section (YouTube, Netflix, Prime Video, Disney+, Hotstar, Spotify)
        self.render_apps_section(ui, w);
        ui.add_space(6.0);
    }

    // -----------------------------------------------------------------------
    // Power & Volume Row
    // -----------------------------------------------------------------------

    fn render_power_volume_row(&mut self, ui: &mut Ui, snap: &Shared, w: f32) {
        let gap = 8.0;
        let power_w = (w * 0.35).floor();
        let vol_w = w - power_w - gap;
        let h = 38.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;

            // Power Button (Red power icon + "Power" text)
            if custom_button(ui, vec2(power_w, h), "Power", BtnIcon::Power, None, 14.0, false) {
                self.ctrl.key(keys::POWER);
            }

            // Volume Card (Speaker icon, Volume 0/0, custom slider bar)
            let (rect, resp) = ui.allocate_exact_size(vec2(vol_w, h), Sense::click_and_drag());
            let bg = if resp.hovered() { Color32::from_rgb(20, 31, 50) } else { CARD_BG };
            let painter = ui.painter();
            painter.rect_filled(rect, 14.0, bg);
            painter.rect_stroke(rect, 14.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);

            // Speaker Icon on left inside card using vector SVG
            let spk_c = pos2(rect.min.x + 20.0, rect.center().y);
            svg_volume_speaker(ui, Rect::from_center_size(spk_c, vec2(15.0, 15.0)), ICON_SLATE);

            // Volume info text and slider
            let text_x = rect.min.x + 38.0;
            let (vol_text, frac) = match snap.volume {
                Some((lvl, max, muted)) => {
                    let f = if max > 0 { (lvl as f32 / max as f32).clamp(0.0, 1.0) } else { 0.0 };
                    let txt = if muted {
                        "Muted".to_string()
                    } else {
                        format!("Volume {lvl}/{max}")
                    };
                    (txt, f)
                }
                None => ("Volume 0/0".to_string(), self.sim_volume),
            };

            // "Volume 0/0" label
            painter.text(
                pos2(text_x, rect.center().y - 7.5),
                Align2::LEFT_CENTER,
                &vol_text,
                FontId::proportional(10.5),
                TEXT_MUTED,
            );

            // Volume track groove
            let track_start = pos2(text_x, rect.center().y + 7.0);
            let track_w = (rect.max.x - 16.0 - text_x).max(40.0);
            let track_rect = Rect::from_min_size(track_start, vec2(track_w, 3.5));
            painter.rect_filled(track_rect, 1.75, Color32::from_rgb(10, 16, 26));

            // Filled electric blue track
            let filled_w = track_w * frac;
            if filled_w > 0.0 {
                let filled_rect = Rect::from_min_size(track_start, vec2(filled_w, 3.5));
                painter.rect_filled(filled_rect, 1.75, ACCENT);
            }

            // Blue thumb knob
            let knob_x = track_start.x + filled_w;
            let knob_c = pos2(knob_x, track_start.y + 1.75);
            painter.circle_filled(knob_c, 4.5, ACCENT);

            // Drag / click volume adjustment with tactile ratchet haptics
            if resp.dragged() || resp.clicked() {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    let new_f = ((pos.x - track_start.x) / track_w).clamp(0.0, 1.0);
                    if (new_f - self.sim_volume).abs() >= 0.04 {
                        haptics::trigger(HapticStyle::Tick);
                        if new_f > self.sim_volume {
                            self.ctrl.key(keys::VOLUME_UP);
                        } else if new_f < self.sim_volume {
                            self.ctrl.key(keys::VOLUME_DOWN);
                        }
                        self.sim_volume = new_f;
                    }
                }
            }
        });
    }

    // -----------------------------------------------------------------------
    // Futuristic Clover D-Pad Component
    // -----------------------------------------------------------------------

    fn render_dpad(&mut self, ui: &mut Ui, w: f32) {
        let dpad_total_size = 158.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(w, dpad_total_size), Sense::click_and_drag());
        let center = rect.center();
        let dist = 46.0;
        let r_lobe = 23.0;
        let r_fillet = 16.0;
        let center_r = 29.0;
        let pad_r = 21.0;

        let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));
        let mut hovered_dir = None;
        let mut center_hovered = false;

        if let Some(pos) = pointer_pos {
            let delta = pos - center;
            let d = delta.length();
            if d <= center_r {
                center_hovered = true;
            } else {
                let dirs = [
                    (Dir::Up, center + vec2(0.0, -dist)),
                    (Dir::Down, center + vec2(0.0, dist)),
                    (Dir::Left, center + vec2(-dist, 0.0)),
                    (Dir::Right, center + vec2(dist, 0.0)),
                ];
                for (dir, lobe_c) in dirs {
                    if (pos - lobe_c).length() <= pad_r + 6.0 {
                        hovered_dir = Some(dir);
                        break;
                    }
                }
                if hovered_dir.is_none() && d <= dist + r_lobe + 6.0 {
                    let angle = delta.y.atan2(delta.x);
                    if angle >= -std::f32::consts::FRAC_PI_4 && angle < std::f32::consts::FRAC_PI_4 {
                        hovered_dir = Some(Dir::Right);
                    } else if angle >= std::f32::consts::FRAC_PI_4 && angle < 3.0 * std::f32::consts::FRAC_PI_4 {
                        hovered_dir = Some(Dir::Down);
                    } else if angle >= -3.0 * std::f32::consts::FRAC_PI_4 && angle < -std::f32::consts::FRAC_PI_4 {
                        hovered_dir = Some(Dir::Up);
                    } else {
                        hovered_dir = Some(Dir::Left);
                    }
                }
            }
        }

        let is_down = resp.is_pointer_button_down_on();
        let pressed_dir = if is_down { hovered_dir } else { None };
        let center_pressed = is_down && center_hovered;

        let fire = hold_logic(ui, &resp);
        if fire {
            haptics::trigger(HapticStyle::Heavy);
            if center_pressed {
                self.ctrl.key(keys::DPAD_CENTER);
            } else if let Some(dir) = pressed_dir {
                match dir {
                    Dir::Up => self.ctrl.key(keys::DPAD_UP),
                    Dir::Down => self.ctrl.key(keys::DPAD_DOWN),
                    Dir::Left => self.ctrl.key(keys::DPAD_LEFT),
                    Dir::Right => self.ctrl.key(keys::DPAD_RIGHT),
                }
            }
        }

        draw_clover_dpad(
            ui,
            center,
            dist,
            r_lobe,
            r_fillet,
            hovered_dir,
            pressed_dir,
            center_hovered,
            center_pressed,
        );
    }

    // -----------------------------------------------------------------------
    // Navigation Row (Back, Home, Menu)
    // -----------------------------------------------------------------------

    fn render_nav_row(&mut self, ui: &mut Ui, w: f32) {
        let gap = 7.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Back", BtnIcon::Back, None, 14.0, false) {
                self.ctrl.key(keys::BACK);
            }
            if custom_button(ui, vec2(btn_w, h), "Home", BtnIcon::Home, None, 14.0, false) {
                self.ctrl.key(keys::HOME);
            }
            if custom_button(ui, vec2(btn_w, h), "Menu", BtnIcon::Menu, None, 14.0, false) {
                self.ctrl.key(keys::MENU);
            }
        });
    }

    // -----------------------------------------------------------------------
    // Function Rows
    // -----------------------------------------------------------------------

    fn render_function_row_1(&mut self, ui: &mut Ui, w: f32) {
        let gap = 6.5;
        let btn_w = (w - 3.0 * gap) / 4.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Vol -", BtnIcon::VolMinus, None, 12.0, true) {
                self.ctrl.key(keys::VOLUME_DOWN);
            }
            if custom_button(ui, vec2(btn_w, h), "Mute", BtnIcon::Mute, None, 12.0, false) {
                self.ctrl.key(keys::VOLUME_MUTE);
            }
            if custom_button(ui, vec2(btn_w, h), "Vol +", BtnIcon::VolPlus, None, 12.0, true) {
                self.ctrl.key(keys::VOLUME_UP);
            }
            if custom_button(ui, vec2(btn_w, h), "Input", BtnIcon::Input, None, 12.0, false) {
                self.ctrl.key(keys::TV_INPUT);
            }
        });
    }

    fn render_function_row_2(&mut self, ui: &mut Ui, w: f32) {
        let gap = 6.5;
        let btn_w = (w - 3.0 * gap) / 4.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Ch -", BtnIcon::ChevronDown, None, 12.0, true) {
                self.ctrl.key(keys::CHANNEL_DOWN);
            }
            if custom_button(ui, vec2(btn_w, h), "Ch +", BtnIcon::ChevronUp, None, 12.0, true) {
                self.ctrl.key(keys::CHANNEL_UP);
            }
            if custom_button(ui, vec2(btn_w, h), "Guide", BtnIcon::Guide, None, 12.0, false) {
                self.ctrl.key(keys::GUIDE);
            }
            if custom_button(ui, vec2(btn_w, h), "Info", BtnIcon::Info, None, 12.0, false) {
                self.ctrl.key(keys::INFO);
            }
        });
    }

    fn render_function_row_3(&mut self, ui: &mut Ui, w: f32) {
        let gap = 7.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 36.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button_stacked(ui, vec2(btn_w, h), "Prev", BtnIcon::Prev, None, 12.0, true) {
                self.ctrl.key(keys::MEDIA_PREVIOUS);
            }
            if custom_button_stacked(ui, vec2(btn_w, h), "Play/Pause", BtnIcon::PlayPause, None, 12.0, false) {
                self.ctrl.key(keys::MEDIA_PLAY_PAUSE);
            }
            if custom_button_stacked(ui, vec2(btn_w, h), "Next", BtnIcon::Next, None, 12.0, true) {
                self.ctrl.key(keys::MEDIA_NEXT);
            }
        });
    }

    fn render_function_row_4(&mut self, ui: &mut Ui, w: f32) {
        let gap = 7.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Search", BtnIcon::Search, None, 12.0, false) {
                self.ctrl.key(keys::SEARCH);
            }
            if custom_button(ui, vec2(btn_w, h), "Captions", BtnIcon::Captions, None, 12.0, false) {
                self.ctrl.key(keys::CAPTIONS);
            }
            if custom_button(ui, vec2(btn_w, h), "Settings", BtnIcon::Settings, None, 12.0, false) {
                self.ctrl.key(keys::SETTINGS);
            }
        });
    }

    // -----------------------------------------------------------------------
    // Apps Section (3 Columns x 2 Rows)
    // -----------------------------------------------------------------------

    fn render_apps_section(&mut self, ui: &mut Ui, w: f32) {
        ui.label(RichText::new("Apps").size(13.0).strong().color(TEXT_WHITE));
        ui.add_space(4.0);

        let gap = 7.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 34.0;

        // Row 1: YouTube, Netflix, Prime Video
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "YouTube", BtnIcon::YouTube, None, 12.0, false) {
                self.ctrl.launch("https://www.youtube.com");
            }
            if custom_button(ui, vec2(btn_w, h), "Netflix", BtnIcon::Netflix, None, 12.0, false) {
                self.ctrl.launch("https://www.netflix.com/title");
            }
            if custom_button(ui, vec2(btn_w, h), "Prime Video", BtnIcon::Prime, None, 12.0, false) {
                self.ctrl.launch("https://app.primevideo.com");
            }
        });

        ui.add_space(4.0);

        // Row 2: Disney+, Hotstar, Spotify
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Disney+", BtnIcon::Disney, None, 12.0, false) {
                self.ctrl.launch("https://www.disneyplus.com");
            }
            if custom_button(ui, vec2(btn_w, h), "Hotstar", BtnIcon::Hotstar, None, 12.0, false) {
                self.ctrl.launch("https://www.hotstar.com");
            }
            if custom_button(ui, vec2(btn_w, h), "Spotify", BtnIcon::Spotify, None, 12.0, false) {
                self.ctrl.launch("spotify://");
            }
        });
    }

    // -----------------------------------------------------------------------
    // Dedicated Apps Tab
    // -----------------------------------------------------------------------

    fn render_apps_tab(&mut self, ui: &mut Ui, w: f32) {
        ui.heading(RichText::new("All TV Apps").color(TEXT_WHITE));
        ui.label(RichText::new("Quick launch any streaming service on your Google TV").color(TEXT_MUTED).size(12.0));
        ui.add_space(12.0);

        let apps: &[(&str, BtnIcon, &str)] = &[
            ("YouTube", BtnIcon::YouTube, "https://www.youtube.com"),
            ("Netflix", BtnIcon::Netflix, "https://www.netflix.com/title"),
            ("Prime", BtnIcon::Prime, "https://app.primevideo.com"),
            ("Disney+", BtnIcon::Disney, "https://www.disneyplus.com"),
            ("Hotstar", BtnIcon::Hotstar, "https://www.hotstar.com"),
            ("Spotify", BtnIcon::Spotify, "spotify://"),
            ("Apple TV", BtnIcon::None, "https://tv.apple.com"),
            ("Twitch", BtnIcon::None, "https://www.twitch.tv"),
            ("Plex", BtnIcon::None, "plex://"),
            ("Crunchyroll", BtnIcon::None, "https://www.crunchyroll.com"),
            ("Hulu", BtnIcon::None, "https://www.hulu.com"),
            ("Tubi", BtnIcon::None, "https://tubitv.com"),
        ];

        let gap = 8.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 50.0;

        for chunk in apps.chunks(3) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (name, icon, link) in chunk {
                    if custom_button(ui, vec2(btn_w, h), name, *icon, None, 12.0, false) {
                        self.ctrl.launch(link);
                    }
                }
            });
            ui.add_space(6.0);
        }
        ui.add_space(60.0);
    }

    // -----------------------------------------------------------------------
    // Dedicated Touchpad Tab
    // -----------------------------------------------------------------------

    fn render_touchpad_tab(&mut self, ui: &mut Ui, w: f32, max_h: f32) {
        ui.heading(RichText::new("Touchpad").color(TEXT_WHITE));
        ui.label(RichText::new("Swipe in any direction to move - Tap anywhere to select").color(TEXT_MUTED).size(12.0));
        ui.add_space(10.0);

        let pad_h = (max_h - 110.0).clamp(200.0, 380.0);
        let (rect, resp) = ui.allocate_exact_size(vec2(w, pad_h), Sense::click_and_drag());
        let painter = ui.painter();
        painter.rect_filled(rect, 20.0, CARD_BG);
        painter.rect_stroke(rect, 20.0, Stroke::new(1.2, BORDER), StrokeKind::Inside);

        // Touchpad icon and instruction in center
        let c = rect.center();
        svg_nav_touchpad(ui, Rect::from_center_size(c + vec2(0.0, -18.0), vec2(28.0, 28.0)), ICON_SLATE);
        painter.text(
            c + vec2(0.0, 14.0),
            Align2::CENTER_CENTER,
            "Swipe to Navigate  •  Tap to Select",
            FontId::proportional(14.0),
            TEXT_MUTED,
        );

        // Gesture handling with tactile tick haptics
        const STEP: f32 = 42.0;
        if resp.dragged() {
            self.touch_acc += resp.drag_delta();
            while self.touch_acc.x > STEP {
                haptics::trigger(HapticStyle::Tick);
                self.ctrl.key(keys::DPAD_RIGHT);
                self.touch_acc.x -= STEP;
            }
            while self.touch_acc.x < -STEP {
                haptics::trigger(HapticStyle::Tick);
                self.ctrl.key(keys::DPAD_LEFT);
                self.touch_acc.x += STEP;
            }
            while self.touch_acc.y > STEP {
                haptics::trigger(HapticStyle::Tick);
                self.ctrl.key(keys::DPAD_DOWN);
                self.touch_acc.y -= STEP;
            }
            while self.touch_acc.y < -STEP {
                haptics::trigger(HapticStyle::Tick);
                self.ctrl.key(keys::DPAD_UP);
                self.touch_acc.y += STEP;
            }
        } else {
            self.touch_acc = Vec2::ZERO;
        }

        // Tactile touch feedback: glowing neon ring under pointer during interaction
        if let Some(pos) = resp.interact_pointer_pos() {
            painter.circle_filled(pos, 24.0, Color32::from_rgba_premultiplied(59, 130, 246, 35));
            painter.circle_filled(pos, 10.0, Color32::from_rgba_premultiplied(56, 189, 248, 80));
            painter.circle_stroke(pos, 24.0, Stroke::new(1.5, ACCENT));
        }

        if resp.clicked() {
            haptics::trigger(HapticStyle::Medium);
            self.ctrl.key(keys::DPAD_CENTER);
        }

        ui.add_space(14.0);
        // Quick Back and Home buttons under trackpad
        let gap = 8.0;
        let btn_w = (w - gap) / 2.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, 48.0), "Back", BtnIcon::Back, None, 14.0, false) {
                self.ctrl.key(keys::BACK);
            }
            if custom_button(ui, vec2(btn_w, 48.0), "Home", BtnIcon::Home, None, 14.0, false) {
                self.ctrl.key(keys::HOME);
            }
        });
    }

    // -----------------------------------------------------------------------
    // Dedicated More Tab (Virtual Keyboard & Tools)
    // -----------------------------------------------------------------------

    fn render_more_tab(&mut self, ui: &mut Ui, snap: &Shared, w: f32, next: &mut Option<Screen>) {
        ui.heading(RichText::new("Tools & Settings").color(TEXT_WHITE));
        ui.add_space(10.0);

        // TV Switcher Button
        if custom_button(ui, vec2(w, 48.0), "Switch / Manage TVs", BtnIcon::Tv, None, 14.0, false) {
            self.show_tv_selector = true;
            self.tab = Tab::Remote;
        }

        ui.add_space(14.0);

        // Feedback & Vibration Settings Card
        egui::Frame::new()
            .fill(CARD_BG)
            .corner_radius(14)
            .stroke(Stroke::new(1.0, BORDER))
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(w - 24.0);
                ui.label(RichText::new("Feedback & Vibration").strong().color(TEXT_WHITE).size(15.0));
                ui.label(RichText::new("Calibrated tactile haptics & native system click sounds").color(TEXT_MUTED).size(11.5));
                ui.add_space(8.0);

                let btn_w = (w - 24.0 - 24.0 - 8.0) / 2.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let vib_active = haptics::is_enabled();
                    let vib_txt = if vib_active { "Vibration: ON" } else { "Vibration: OFF" };
                    let vib_bg = if vib_active { Some(Color32::from_rgb(18, 52, 40)) } else { None };
                    if custom_button(ui, vec2(btn_w, 36.0), vib_txt, BtnIcon::None, vib_bg, 10.0, false) {
                        let next = !vib_active;
                        haptics::set_enabled(next);
                        if next {
                            haptics::trigger(HapticStyle::Heavy);
                        }
                    }

                    let snd_active = haptics::is_sound_enabled();
                    let snd_txt = if snd_active { "Sound: ON" } else { "Sound: OFF" };
                    let snd_bg = if snd_active { Some(Color32::from_rgb(18, 52, 40)) } else { None };
                    if custom_button(ui, vec2(btn_w, 36.0), snd_txt, BtnIcon::None, snd_bg, 10.0, false) {
                        let next = !snd_active;
                        haptics::set_sound_enabled(next);
                        if next {
                            haptics::trigger(HapticStyle::Light);
                        }
                    }
                });

                ui.add_space(8.0);
                let test_w = w - 48.0;
                if custom_button(ui, vec2(test_w, 36.0), "⚡ Test Tactile Vibration", BtnIcon::None, Some(Color32::from_rgb(26, 42, 68)), 10.0, false) {
                    haptics::trigger(HapticStyle::Heavy);
                }
            });

        ui.add_space(14.0);

        // Virtual Keyboard
        ui.label(RichText::new("Type on TV").size(15.0).strong().color(TEXT_WHITE));
        ui.label(RichText::new("Tap letters to enter text into search or input fields").color(TEXT_MUTED).size(11.5));
        ui.add_space(6.0);

        let gap = 4.0;
        let kw = ((w - 9.0 * gap) / 10.0).floor();
        for row in ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"] {
            let width = row.len() as f32 * kw + (row.len() as f32 - 1.0) * gap;
            centered(ui, width, |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for c in row.chars() {
                    if custom_button(ui, vec2(kw, 42.0), &c.to_string(), BtnIcon::None, None, 8.0, false) {
                        if let Some(k) = keys::key_for_char(c) {
                            self.ctrl.key(k);
                        }
                    }
                }
            });
            ui.add_space(4.0);
        }

        // Punctuation and Space / Delete / Enter
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            let small = vec2(kw * 1.4, 42.0);
            for (label, c) in [(".", '.'), ("@", '@'), ("-", '-'), ("/", '/')] {
                if custom_button(ui, small, label, BtnIcon::None, None, 8.0, false) {
                    if let Some(k) = keys::key_for_char(c) {
                        self.ctrl.key(k);
                    }
                }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(w * 0.40, 46.0), "Space", BtnIcon::None, None, 10.0, false) {
                self.ctrl.key(keys::SPACE);
            }
            if custom_button(ui, vec2(w * 0.26, 46.0), "Delete", BtnIcon::None, None, 10.0, true) {
                self.ctrl.key(keys::DEL);
            }
            if custom_button(ui, vec2(w * 0.28, 46.0), "Enter", BtnIcon::None, Some(ACCENT), 10.0, false) {
                self.ctrl.key(keys::ENTER);
            }
        });

        ui.add_space(20.0);

        // Connection Diagnostics
        egui::Frame::new()
            .fill(CARD_BG)
            .corner_radius(14)
            .stroke(Stroke::new(1.0, BORDER))
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(w - 24.0);
                ui.label(RichText::new("Connection Info").strong().color(TEXT_WHITE));
                let status_text = match &snap.status {
                    ConnStatus::Connected => "Connected to TV",
                    ConnStatus::Connecting => "Connecting...",
                    ConnStatus::Failed(e) => e.as_str(),
                    ConnStatus::Idle => "Idle",
                };
                ui.label(RichText::new(format!("Status: {status_text}")).color(TEXT_MUTED));

                if let Some(tv) = &self.ctrl.active {
                    ui.label(RichText::new(format!("TV: {} ({})", tv.name, tv.host)).color(TEXT_MUTED));
                }

                ui.horizontal(|ui| {
                    if ui.button("Disconnect").clicked() {
                        self.ctrl.disconnect();
                    }
                    if let Some(tv) = self.ctrl.active.clone() {
                        if ui.button("Re-Pair").clicked() {
                            self.ctrl.disconnect();
                            self.code_input.clear();
                            self.ctrl.start_pairing(&tv);
                            *next = Some(Screen::Pair(tv));
                        }
                    }
                });
            });

        ui.add_space(70.0);
    }

    // -----------------------------------------------------------------------
    // Pairing Screen
    // -----------------------------------------------------------------------

    fn pair_screen(&mut self, ui: &mut Ui, snap: &Shared, tv: &Tv, next: &mut Option<Screen>) {
        ui.add_space(10.0);
        ui.heading(RichText::new(format!("Pair with {}", tv.name)).color(TEXT_WHITE));
        ui.label(RichText::new("A 6-character code is showing on your TV screen.").color(TEXT_MUTED).size(13.0));
        ui.add_space(12.0);

        match &snap.pairing {
            PairingUi::None | PairingUi::Starting => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Connecting to the TV...");
                });
            }
            PairingUi::WaitingForCode | PairingUi::WrongCode | PairingUi::Verifying => {
                if snap.pairing == PairingUi::WrongCode {
                    ui.colored_label(Color32::from_rgb(255, 95, 95), "That code doesn't match. Check the TV and try again.");
                }
                let shown = format!("{:_<6}", self.code_input.to_uppercase());
                ui.label(RichText::new(shown).size(38.0).monospace().strong().color(ACCENT));
                keypad(ui, &mut self.code_input, true, 6);

                let verifying = snap.pairing == PairingUi::Verifying;
                if verifying {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Verifying code...");
                    });
                }
                let ready = self.code_input.len() == 6 && !verifying;
                if ui.add_enabled(ready, egui::Button::new(RichText::new("Submit Code").size(16.0)).fill(ACCENT)).clicked() {
                    self.ctrl.submit_code(&self.code_input);
                }
            }
            PairingUi::Done => {
                ui.colored_label(GREEN_TEXT, "Paired successfully!");
                let mut tv = tv.clone();
                tv.paired = true;
                self.ctrl.store.mark_paired(&tv.host);
                self.ctrl.connect(&tv);
                *next = Some(Screen::Remote);
            }
            PairingUi::Error(e) => {
                ui.colored_label(Color32::from_rgb(255, 95, 95), format!("Pairing failed: {e}"));
                if ui.button("Try again").clicked() {
                    self.code_input.clear();
                    self.ctrl.start_pairing(tv);
                }
            }
        }

        ui.add_space(14.0);
        if ui.button("Cancel").clicked() {
            self.ctrl.cancel_pairing();
            *next = Some(Screen::Remote);
        }
        ui.add_space(70.0);
    }
}

// ---------------------------------------------------------------------------
// Button and Widget Helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BtnIcon {
    None,
    Power,
    Tv,
    ChevronDown,
    ChevronUp,
    Gear,
    Back,
    Home,
    Menu,
    VolMinus,
    VolPlus,
    Mute,
    Input,
    Guide,
    Info,
    Prev,
    PlayPause,
    Next,
    Search,
    Captions,
    Settings,
    YouTube,
    Netflix,
    Prime,
    Disney,
    Hotstar,
    Spotify,
}

fn custom_button(
    ui: &mut Ui,
    size: Vec2,
    label: &str,
    icon: BtnIcon,
    custom_fill: Option<Color32>,
    corner_radius: f32,
    repeat: bool,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fire = if repeat {
        hold_logic(ui, &resp)
    } else {
        resp.clicked()
    };

    let is_down = resp.is_pointer_button_down_on();
    let bg = if is_down {
        custom_fill.unwrap_or(Color32::from_rgb(32, 52, 86))
    } else if resp.hovered() {
        custom_fill.unwrap_or(BTN_HOVER)
    } else {
        custom_fill.unwrap_or(BTN_BG)
    };

    let draw_rect = if is_down {
        rect.shrink(1.2)
    } else {
        rect
    };

    let stroke = if is_down {
        Stroke::new(1.8, Color32::from_rgb(56, 189, 248))
    } else if resp.hovered() {
        Stroke::new(1.0, Color32::from_rgb(45, 68, 108))
    } else {
        Stroke::new(1.0, BORDER)
    };

    let painter = ui.painter();
    painter.rect_filled(draw_rect, corner_radius, bg);
    painter.rect_stroke(draw_rect, corner_radius, stroke, StrokeKind::Inside);

    let c = draw_rect.center();
    let text_color = if custom_fill.is_some() {
        TEXT_WHITE
    } else {
        Color32::from_rgb(226, 232, 240)
    };

    let font_size = if size.x < 75.0 {
        11.5
    } else if size.y < 42.0 {
        12.0
    } else {
        13.0
    };
    let font_id = FontId::proportional(font_size);

    if icon == BtnIcon::None {
        painter.text(c, Align2::CENTER_CENTER, label, font_id, text_color);
    } else if label.is_empty() {
        let icon_box = match icon {
            BtnIcon::Disney => Rect::from_center_size(c, vec2(56.0, 22.0)),
            BtnIcon::YouTube => Rect::from_center_size(c, vec2(22.0, 16.0)),
            BtnIcon::Netflix => Rect::from_center_size(c, vec2(16.0, 16.0)),
            BtnIcon::Prime => Rect::from_center_size(c, vec2(30.0, 18.0)),
            _ => Rect::from_center_size(c, vec2(16.0, 16.0)),
        };
        render_btn_svg(ui, icon_box, icon, text_color);
    } else {
        let text_layout = painter.layout_no_wrap(label.to_string(), font_id.clone(), text_color);
        let text_w = text_layout.size().x;
        let (icon_w, icon_h) = match icon {
            BtnIcon::YouTube => (19.0, 14.0),
            BtnIcon::Netflix => (14.0, 15.0),
            BtnIcon::Prime => (24.0, 15.0),
            BtnIcon::Hotstar => (15.0, 15.0),
            BtnIcon::Spotify => (15.0, 15.0),
            _ => (15.0, 15.0),
        };
        let gap = 5.0;
        let total_w = icon_w + gap + text_w;
        let start_x = c.x - total_w * 0.5;

        let icon_rect = Rect::from_center_size(pos2(start_x + icon_w * 0.5, c.y), vec2(icon_w, icon_h));
        render_btn_svg(ui, icon_rect, icon, text_color);

        let text_p = pos2(start_x + icon_w + gap, c.y);
        painter.text(text_p, Align2::LEFT_CENTER, label, font_id, text_color);
    }

    if fire {
        let style = match icon {
            BtnIcon::Power => HapticStyle::Heavy,
            BtnIcon::None => HapticStyle::Light,
            _ => HapticStyle::Medium,
        };
        haptics::trigger(style);
    }

    fire
}

fn custom_button_stacked(
    ui: &mut Ui,
    size: Vec2,
    label: &str,
    icon: BtnIcon,
    custom_fill: Option<Color32>,
    corner_radius: f32,
    repeat: bool,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fire = if repeat {
        hold_logic(ui, &resp)
    } else {
        resp.clicked()
    };

    let is_down = resp.is_pointer_button_down_on();
    let bg = if is_down {
        custom_fill.unwrap_or(Color32::from_rgb(32, 52, 86))
    } else if resp.hovered() {
        custom_fill.unwrap_or(BTN_HOVER)
    } else {
        custom_fill.unwrap_or(BTN_BG)
    };

    let draw_rect = if is_down {
        rect.shrink(1.2)
    } else {
        rect
    };

    let stroke = if is_down {
        Stroke::new(1.8, Color32::from_rgb(56, 189, 248))
    } else if resp.hovered() {
        Stroke::new(1.0, Color32::from_rgb(45, 68, 108))
    } else {
        Stroke::new(1.0, BORDER)
    };

    let painter = ui.painter();
    painter.rect_filled(draw_rect, corner_radius, bg);
    painter.rect_stroke(draw_rect, corner_radius, stroke, StrokeKind::Inside);

    let c = draw_rect.center();
    let text_color = if custom_fill.is_some() {
        TEXT_WHITE
    } else {
        Color32::from_rgb(226, 232, 240)
    };

    let icon_rect = Rect::from_center_size(pos2(c.x, c.y - 7.0), vec2(16.0, 13.0));
    render_btn_svg(ui, icon_rect, icon, text_color);

    let font_id = FontId::proportional(10.0);
    painter.text(pos2(c.x, c.y + 7.5), Align2::CENTER_CENTER, label, font_id, text_color);

    if fire {
        let style = match icon {
            BtnIcon::PlayPause => HapticStyle::Heavy,
            _ => HapticStyle::Medium,
        };
        haptics::trigger(style);
    }

    fire
}

fn render_btn_svg(ui: &Ui, rect: Rect, icon: BtnIcon, color: Color32) {
    match icon {
        BtnIcon::None => {}
        BtnIcon::Power => svg_power(ui, rect),
        BtnIcon::Tv => svg_tv(ui, rect, color),
        BtnIcon::ChevronDown => svg_chevron_down(ui, rect, color),
        BtnIcon::ChevronUp => svg_chevron_up(ui, rect, color),
        BtnIcon::Gear | BtnIcon::Settings => svg_settings(ui, rect, color),
        BtnIcon::Back => svg_back(ui, rect, color),
        BtnIcon::Home => svg_home(ui, rect, color),
        BtnIcon::Menu => svg_menu(ui, rect, color),
        BtnIcon::VolMinus => svg_vol_down(ui, rect, color),
        BtnIcon::VolPlus => svg_vol_up(ui, rect, color),
        BtnIcon::Mute => svg_vol_mute(ui, rect),
        BtnIcon::Input => svg_input(ui, rect, color),
        BtnIcon::Guide => svg_guide(ui, rect, color),
        BtnIcon::Info => svg_info(ui, rect, color),
        BtnIcon::Prev => svg_prev(ui, rect, color),
        BtnIcon::PlayPause => svg_play_pause(ui, rect, color),
        BtnIcon::Next => svg_next(ui, rect, color),
        BtnIcon::Search => svg_search(ui, rect, color),
        BtnIcon::Captions => svg_captions(ui, rect, color),
        BtnIcon::YouTube => svg_youtube(ui, rect),
        BtnIcon::Netflix => svg_netflix(ui, rect),
        BtnIcon::Prime => svg_prime(ui, rect),
        BtnIcon::Disney => svg_disney(ui, rect),
        BtnIcon::Hotstar => svg_hotstar(ui, rect),
        BtnIcon::Spotify => svg_spotify(ui, rect),
    }
}

// ---------------------------------------------------------------------------
// Bottom Navigation Bar
// ---------------------------------------------------------------------------

fn bottom_nav_bar(ui: &mut Ui, current_tab: &mut Tab, w: f32) {
    let bar_h = 50.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, bar_h), Sense::hover());
    let painter = ui.painter();

    // Dark bar background and top border line
    painter.rect_filled(rect, 0.0, Color32::from_rgb(9, 13, 22));
    painter.line_segment(
        [rect.min, pos2(rect.max.x, rect.min.y)],
        Stroke::new(1.0, Color32::from_rgb(22, 34, 52)),
    );

    let items = [
        (Tab::Remote, "Remote"),
        (Tab::Apps, "Apps"),
        (Tab::Touchpad, "Touchpad"),
        (Tab::More, "More"),
    ];

    let item_w = w / 4.0;
    for (i, &(tab, label)) in items.iter().enumerate() {
        let item_rect = Rect::from_min_size(
            pos2(rect.min.x + (i as f32) * item_w, rect.min.y),
            vec2(item_w, bar_h),
        );
        let resp = ui.interact(item_rect, ui.scope_id().with(i), Sense::click());
        if resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            *current_tab = tab;
        }

        let is_active = *current_tab == tab;
        let color = if is_active { ACCENT } else { TEXT_MUTED };

        // Vector SVG Nav Icon with subtle press compression
        let icon_size = if resp.is_pointer_button_down_on() { vec2(14.0, 14.0) } else { vec2(16.0, 16.0) };
        let icon_rect = Rect::from_center_size(pos2(item_rect.center().x, item_rect.min.y + 15.0), icon_size);
        match tab {
            Tab::Remote => svg_nav_remote(ui, icon_rect, color),
            Tab::Apps => svg_nav_apps(ui, icon_rect, color),
            Tab::Touchpad => svg_nav_touchpad(ui, icon_rect, color),
            Tab::More => svg_nav_more(ui, icon_rect, color),
        }

        // Text label
        let text_c = pos2(item_rect.center().x, item_rect.min.y + 33.0);
        painter.text(
            text_c,
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0),
            color,
        );

        // Active blue underline indicator
        if is_active {
            let line_c = pos2(item_rect.center().x, item_rect.max.y - 3.5);
            painter.rect_filled(
                Rect::from_center_size(line_c, vec2(24.0, 2.5)),
                1.2,
                ACCENT,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn centered(ui: &mut Ui, content_width: f32, f: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        let pad = ((ui.available_width() - content_width) / 2.0).max(0.0);
        ui.add_space(pad);
        f(ui);
    });
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

fn keypad(ui: &mut Ui, text: &mut String, hex: bool, max_len: usize) {
    let rows: &[&str] = if hex {
        &["0123", "4567", "89AB", "CDEF"]
    } else {
        &["123", "456", "789", ".0"]
    };
    let gap = 6.0;
    let cols = if hex { 4.0 } else { 3.0 };
    let w = ((ui.available_width() - gap * (cols - 1.0)) / cols).min(84.0);
    for row in rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for c in row.chars() {
                if custom_button(ui, vec2(w, 48.0), &c.to_string(), BtnIcon::None, None, 10.0, false)
                    && text.len() < max_len
                {
                    text.push(c);
                }
            }
            if !hex && *row == ".0" && custom_button(ui, vec2(w, 48.0), "Del", BtnIcon::None, None, 10.0, false) {
                text.pop();
            }
        });
        ui.add_space(4.0);
    }
    if hex {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(w * 2.0 + gap, 44.0), "Del", BtnIcon::None, None, 10.0, false) {
                text.pop();
            }
            if custom_button(ui, vec2(w * 2.0 + gap, 44.0), "Clear", BtnIcon::None, None, 10.0, false) {
                text.clear();
            }
        });
    }
}
