//! The egui user interface for FryRemote: exact 1:1 match with modern Android TV remote design.

use crate::controller::{Controller, Notify};
use crate::icons::*;
use crate::keys;
use crate::state::*;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Theme Colors (matched to the design mock)
// ---------------------------------------------------------------------------

pub const BG: Color32 = Color32::from_rgb(9, 14, 24); // #090e18 deep midnight navy
pub const CARD_BG: Color32 = Color32::from_rgb(17, 27, 43); // #111b2b sleek card surface
pub const BTN_BG: Color32 = Color32::from_rgb(20, 33, 53); // #142135 button surface
pub const BTN_HOVER: Color32 = Color32::from_rgb(28, 46, 74); // #1c2e4a
pub const BTN_ACTIVE: Color32 = Color32::from_rgb(36, 60, 96); // #243c60
pub const BORDER: Color32 = Color32::from_rgb(26, 40, 64); // #1a2840 subtle border
pub const ACCENT: Color32 = Color32::from_rgb(41, 121, 255); // #2979ff vivid electric blue
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(65, 138, 255);
pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(26, 95, 220);
pub const GREEN_TEXT: Color32 = Color32::from_rgb(0, 230, 118); // #00e676 connected green
pub const GREEN_BG: Color32 = Color32::from_rgb(12, 34, 30); // #0c221e
pub const GREEN_BORDER: Color32 = Color32::from_rgb(18, 62, 50); // #123e32
pub const RED_POWER: Color32 = Color32::from_rgb(255, 77, 77); // #ff4d4d
pub const TEXT_WHITE: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(112, 130, 153); // #708299 slate secondary
pub const ICON_SLATE: Color32 = Color32::from_rgb(143, 160, 181); // #8fa0b5

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
    typed_text: String,
    show_ip_pad: bool,
    touch_acc: Vec2,
    auto_scanned: bool,
}

impl RemoteApp {
    pub fn new(cc: &eframe::CreationContext<'_>, data_dir: PathBuf) -> Self {
        let ctx = cc.egui_ctx.clone();
        let notify: Notify = Arc::new(move || ctx.request_repaint());
        setup_style(&cc.egui_ctx);
        let mut app = Self {
            ctrl: Controller::new(data_dir, notify),
            screen: Screen::Remote,
            tab: Tab::Remote,
            show_tv_selector: false,
            ip_input: String::new(),
            code_input: String::new(),
            typed_text: String::new(),
            show_ip_pad: false,
            touch_acc: Vec2::ZERO,
            auto_scanned: false,
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
    style.visuals.override_text_color = Some(Color32::from_rgb(240, 246, 255));
    style.visuals.window_fill = BG;
    style.visuals.panel_fill = BG;
    style.visuals.widgets.inactive.bg_fill = BTN_BG;
    style.visuals.widgets.inactive.weak_bg_fill = BTN_BG;
    style.visuals.widgets.hovered.bg_fill = BTN_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = BTN_HOVER;
    style.visuals.widgets.active.bg_fill = BTN_ACTIVE;
    style.visuals.widgets.active.weak_bg_fill = BTN_ACTIVE;
    style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.button_padding = vec2(10.0, 8.0);
    for (_, f) in style.text_styles.iter_mut() {
        f.size = (f.size * 1.15).max(15.0);
    }
    ctx.set_global_style(style);
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

        // Physical keyboard event listener: captures key presses and sends to Android TV
        let has_text_focus = ui.memory(|m| m.focused().is_some());
        if !has_text_focus && !self.show_ip_pad && !matches!(self.screen, Screen::Pair(_)) {
            let events = ui.input(|i| i.events.clone());
            for event in events {
                match event {
                    egui::Event::Text(t) => {
                        if !t.is_empty() {
                            self.ctrl.send_text(&t);
                        }
                    }
                    egui::Event::Key { key, pressed: true, .. } => {
                        match key {
                            egui::Key::Backspace => {
                                self.ctrl.key(keys::DEL);
                            }
                            egui::Key::Enter => {
                                self.ctrl.key(keys::ENTER);
                            }
                            egui::Key::Space => {
                                self.ctrl.send_text(" ");
                            }
                            egui::Key::ArrowUp => {
                                self.ctrl.key(keys::DPAD_UP);
                            }
                            egui::Key::ArrowDown => {
                                self.ctrl.key(keys::DPAD_DOWN);
                            }
                            egui::Key::ArrowLeft => {
                                self.ctrl.key(keys::DPAD_LEFT);
                            }
                            egui::Key::ArrowRight => {
                                self.ctrl.key(keys::DPAD_RIGHT);
                            }
                            egui::Key::Escape => {
                                self.ctrl.key(keys::BACK);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        // Main Central Panel
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(egui::Margin::symmetric(14, 6)))
            .show(ui, |ui| {
                let w = ui.available_width();

                // Header: Logo, FryRemote title, Settings gear
                self.render_header(ui, w);
                ui.add_space(6.0);

                // Main content area based on screen and active tab
                match self.screen.clone() {
                    Screen::Pair(tv) => {
                        self.pair_screen(ui, &snap, &tv, &mut next);
                    }
                    Screen::Remote => {
                        // Compact TV device selector on top-left + Status badge on right
                        self.render_compact_device_row(ui, &snap, w);
                        ui.add_space(6.0);

                        // If TV selector drawer is open
                        if self.show_tv_selector {
                            self.render_tv_selector_drawer(ui, &snap, w, &mut next);
                            ui.add_space(8.0);
                        }

                        let nav_h = 54.0;
                        let scroll_h = (ui.available_height() - nav_h - 4.0).max(100.0);

                        // Tab Content
                        match self.tab {
                            Tab::Remote => {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .max_height(scroll_h)
                                    .show(ui, |ui| {
                                        self.render_remote_tab(ui, &snap, w, &mut next);
                                    });
                            }
                            Tab::Apps => {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
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
                                    .max_height(scroll_h)
                                    .show(ui, |ui| {
                                        self.render_more_tab(ui, &snap, w, &mut next);
                                    });
                            }
                        }

                        // Fixed Bottom Navigation Bar at the bottom
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
// Header Component
// ---------------------------------------------------------------------------

impl RemoteApp {
    fn render_header(&mut self, ui: &mut Ui, _w: f32) {
        ui.horizontal(|ui| {
            // Brand Logo: Controller with blue Wi-Fi waves
            let (logo_rect, _) = ui.allocate_exact_size(vec2(28.0, 34.0), Sense::hover());
            draw_brand_logo(ui.painter(), logo_rect.center());

            ui.add_space(6.0);

            // Title: "Fry" (white) + "Remote" (electric blue)
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.5;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label(RichText::new("Fry").size(21.0).strong().color(TEXT_WHITE));
                    ui.label(RichText::new("Remote").size(21.0).strong().color(ACCENT));
                });
                ui.label(RichText::new("Your Google TV Remote").size(11.0).color(TEXT_MUTED));
            });

            // Right side: Settings Gear button
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (gear_rect, gear_resp) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                let gear_color = if gear_resp.hovered() { TEXT_WHITE } else { ICON_SLATE };
                draw_gear(ui.painter(), gear_rect.center(), 10.0, gear_color);
                if gear_resp.clicked() {
                    self.show_tv_selector = !self.show_tv_selector;
                }
            });
        });
    }

    // -----------------------------------------------------------------------
    // Compact Device Row (Top-Left device pill + Top-Right status pill)
    // -----------------------------------------------------------------------

    fn render_compact_device_row(&mut self, ui: &mut Ui, snap: &Shared, w: f32) {
        let row_h = 30.0;
        let (row_rect, _) = ui.allocate_exact_size(vec2(w, row_h), Sense::hover());
        let painter = ui.painter();

        // Device name logic: Show connected TV name when connected; otherwise "No Device"
        let (device_name, is_connected) = match &snap.status {
            ConnStatus::Connected => {
                if let Some(tv) = &self.ctrl.active {
                    (tv.name.clone(), true)
                } else {
                    ("No Device".to_string(), false)
                }
            }
            ConnStatus::Connecting => ("Connecting...".to_string(), false),
            _ => ("No Device".to_string(), false),
        };

        // 1. Power Button on the left of device button (icon-only, no text)
        let power_w = 34.0;
        let power_rect = Rect::from_min_size(row_rect.min, vec2(power_w, row_h));
        let power_resp = ui.interact(power_rect, ui.scope_id().with("power_btn_top"), Sense::click());
        let power_bg = if power_resp.is_pointer_button_down_on() {
            Color32::from_rgb(50, 18, 24)
        } else if power_resp.hovered() {
            Color32::from_rgb(38, 16, 20)
        } else {
            CARD_BG
        };
        painter.rect_filled(power_rect, 15.0, power_bg);
        painter.rect_stroke(power_rect, 15.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);
        draw_power(painter, power_rect.center(), 7.0, RED_POWER);
        if power_resp.clicked() {
            self.ctrl.key(keys::POWER);
        }

        // 2. Compact Device Selector Pill (to the right of Power Button)
        let pill_start_x = row_rect.min.x + power_w + 6.0;
        let badge_w = 96.0;
        let pill_w = (row_rect.max.x - badge_w - 6.0 - pill_start_x).max(110.0);
        let pill_rect = Rect::from_min_size(pos2(pill_start_x, row_rect.min.y), vec2(pill_w, row_h));
        let resp = ui.interact(pill_rect, ui.scope_id().with("device_pill"), Sense::click());
        let bg = if resp.is_pointer_button_down_on() {
            BTN_ACTIVE
        } else if resp.hovered() {
            Color32::from_rgb(22, 36, 58)
        } else {
            CARD_BG
        };

        painter.rect_filled(pill_rect, 15.0, bg);
        painter.rect_stroke(pill_rect, 15.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);

        // Mini TV icon in electric blue
        let tv_c = pos2(pill_rect.min.x + 16.0, pill_rect.center().y);
        draw_mini_tv(painter, tv_c, ACCENT);

        // TV Name or "No Device"
        let text_x = pill_rect.min.x + 30.0;
        let max_chars = 11;
        let label = if device_name.chars().count() > max_chars {
            format!("{}…", device_name.chars().take(max_chars - 1).collect::<String>())
        } else {
            device_name
        };
        painter.text(
            pos2(text_x, pill_rect.center().y),
            Align2::LEFT_CENTER,
            &label,
            FontId::proportional(12.5),
            if is_connected { TEXT_WHITE } else { TEXT_MUTED },
        );

        // Small chevron dropdown icon on right of pill
        let chev_c = pos2(pill_rect.max.x - 14.0, pill_rect.center().y);
        draw_chevron_down(painter, chev_c, 3.5, ICON_SLATE);

        if resp.clicked() {
            self.show_tv_selector = !self.show_tv_selector;
        }

        // 3. Status Badge on Top-Right
        let (status_text, dot_color, bg_color, border_color) = match &snap.status {
            ConnStatus::Connected => ("Connected", GREEN_TEXT, GREEN_BG, GREEN_BORDER),
            ConnStatus::Connecting => (
                "Connecting",
                Color32::from_rgb(255, 183, 77),
                Color32::from_rgb(38, 28, 12),
                Color32::from_rgb(70, 50, 20),
            ),
            ConnStatus::Failed(_) => (
                "Offline",
                Color32::from_rgb(255, 95, 95),
                Color32::from_rgb(38, 14, 14),
                Color32::from_rgb(70, 25, 25),
            ),
            ConnStatus::Idle => (
                "Disconnected",
                TEXT_MUTED,
                Color32::from_rgb(18, 25, 38),
                Color32::from_rgb(30, 42, 60),
            ),
        };

        let badge_h = 24.0;
        let badge_rect = Rect::from_center_size(
            pos2(row_rect.max.x - badge_w * 0.5, row_rect.center().y),
            vec2(badge_w, badge_h),
        );
        painter.rect_filled(badge_rect, 12.0, bg_color);
        painter.rect_stroke(badge_rect, 12.0, Stroke::new(1.0, border_color), StrokeKind::Inside);
        painter.circle_filled(pos2(badge_rect.min.x + 12.0, badge_rect.center().y), 3.0, dot_color);
        painter.text(
            pos2(badge_rect.min.x + 22.0, badge_rect.center().y),
            Align2::LEFT_CENTER,
            status_text,
            FontId::proportional(11.0),
            dot_color,
        );
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

    fn render_remote_tab(&mut self, ui: &mut Ui, _snap: &Shared, w: f32, _next: &mut Option<Screen>) {
        // 1. D-Pad Directional Arrows & Center Blank OK Button
        self.render_dpad(ui, w);
        ui.add_space(5.0);

        // 2. Navigation Row (Back, Home, Menu)
        self.render_nav_row(ui, w);
        ui.add_space(4.0);

        // 3. Function Grid Row 1 (Vol -, Mute, Vol +, Input)
        self.render_function_row_1(ui, w);
        ui.add_space(4.0);

        // 4. Function Grid Row 2 (Ch -, Ch +, Guide, Info)
        self.render_function_row_2(ui, w);
        ui.add_space(4.0);

        // 5. Media Grid Row 3 (Prev, Play/Pause, Next)
        self.render_function_row_3(ui, w);
        ui.add_space(4.0);

        // 6. Utility Grid Row 4 (Search, Captions, Settings — icon only)
        self.render_function_row_4(ui, w);
        ui.add_space(6.0);

        // 7. Apps Section (YouTube, Netflix, Prime, Disney+, Hotstar, Spotify)
        self.render_apps_section(ui, w);
        ui.add_space(4.0);
    }

    // -----------------------------------------------------------------------
    // Glowing Cloverleaf Neon D-Pad Component
    // -----------------------------------------------------------------------

    fn render_dpad(&mut self, ui: &mut Ui, w: f32) {
        let pod_h = 160.0;
        let (rect, _) = ui.allocate_exact_size(vec2(w, pod_h), Sense::hover());
        let c = rect.center();
        let painter = ui.painter();

        // 1. Generate the exact mathematical C1 cloverleaf boundary points
        let pts = cloverleaf_points(c);

        // 2. Fill cloverleaf pod with dark navy using a triangle fan from center
        let mut mesh = egui::Mesh::default();
        let center_idx = mesh.vertices.len() as u32;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: c,
            uv: egui::epaint::WHITE_UV,
            color: Color32::from_rgb(11, 18, 30),
        });
        for pt in &pts {
            mesh.vertices.push(egui::epaint::Vertex {
                pos: *pt,
                uv: egui::epaint::WHITE_UV,
                color: Color32::from_rgb(11, 18, 30),
            });
        }
        for i in 0..pts.len() {
            let next = (i + 1) % pts.len();
            mesh.indices.push(center_idx);
            mesh.indices.push(center_idx + 1 + i as u32);
            mesh.indices.push(center_idx + 1 + next as u32);
        }
        painter.add(Shape::mesh(mesh));

        // 3. Glowing electric blue neon outline
        let mut pts_closed = pts;
        pts_closed.push(pts_closed[0]);

        // Outer soft cyan/blue bloom
        painter.add(Shape::line(
            pts_closed.clone(),
            Stroke::new(8.0, Color32::from_rgba_premultiplied(0, 110, 255, 30)),
        ));
        // Mid neon glow
        painter.add(Shape::line(
            pts_closed.clone(),
            Stroke::new(4.5, Color32::from_rgba_premultiplied(10, 145, 255, 85)),
        ));
        // Core electric neon stroke
        painter.add(Shape::line(
            pts_closed,
            Stroke::new(2.4, Color32::from_rgb(0, 162, 255)),
        ));

        // 4. Directional Arrow Buttons in the 4 lobes
        let dist = 44.0;
        let r_btn = 16.0;

        // Up Button
        let p_up = c + vec2(0.0, -dist);
        let rect_up = Rect::from_center_size(p_up, vec2(r_btn * 2.0, r_btn * 2.0));
        let resp_up = ui.interact(rect_up, ui.scope_id().with("dpad_up"), Sense::click());
        if hold_logic(ui, &resp_up) {
            self.ctrl.key(keys::DPAD_UP);
        }
        let bg_up = if resp_up.is_pointer_button_down_on() { BTN_ACTIVE } else if resp_up.hovered() { BTN_HOVER } else { BTN_BG };
        painter.circle_filled(p_up, r_btn, bg_up);
        painter.circle_stroke(p_up, r_btn, Stroke::new(1.0, BORDER));
        draw_arrow_triangle(painter, p_up, Dir::Up, 6.5, Color32::WHITE);

        // Down Button
        let p_down = c + vec2(0.0, dist);
        let rect_down = Rect::from_center_size(p_down, vec2(r_btn * 2.0, r_btn * 2.0));
        let resp_down = ui.interact(rect_down, ui.scope_id().with("dpad_down"), Sense::click());
        if hold_logic(ui, &resp_down) {
            self.ctrl.key(keys::DPAD_DOWN);
        }
        let bg_down = if resp_down.is_pointer_button_down_on() { BTN_ACTIVE } else if resp_down.hovered() { BTN_HOVER } else { BTN_BG };
        painter.circle_filled(p_down, r_btn, bg_down);
        painter.circle_stroke(p_down, r_btn, Stroke::new(1.0, BORDER));
        draw_arrow_triangle(painter, p_down, Dir::Down, 6.5, Color32::WHITE);

        // Left Button
        let p_left = c + vec2(-dist, 0.0);
        let rect_left = Rect::from_center_size(p_left, vec2(r_btn * 2.0, r_btn * 2.0));
        let resp_left = ui.interact(rect_left, ui.scope_id().with("dpad_left"), Sense::click());
        if hold_logic(ui, &resp_left) {
            self.ctrl.key(keys::DPAD_LEFT);
        }
        let bg_left = if resp_left.is_pointer_button_down_on() { BTN_ACTIVE } else if resp_left.hovered() { BTN_HOVER } else { BTN_BG };
        painter.circle_filled(p_left, r_btn, bg_left);
        painter.circle_stroke(p_left, r_btn, Stroke::new(1.0, BORDER));
        draw_arrow_triangle(painter, p_left, Dir::Left, 6.5, Color32::WHITE);

        // Right Button
        let p_right = c + vec2(dist, 0.0);
        let rect_right = Rect::from_center_size(p_right, vec2(r_btn * 2.0, r_btn * 2.0));
        let resp_right = ui.interact(rect_right, ui.scope_id().with("dpad_right"), Sense::click());
        if hold_logic(ui, &resp_right) {
            self.ctrl.key(keys::DPAD_RIGHT);
        }
        let bg_right = if resp_right.is_pointer_button_down_on() { BTN_ACTIVE } else if resp_right.hovered() { BTN_HOVER } else { BTN_BG };
        painter.circle_filled(p_right, r_btn, bg_right);
        painter.circle_stroke(p_right, r_btn, Stroke::new(1.0, BORDER));
        draw_arrow_triangle(painter, p_right, Dir::Right, 6.5, Color32::WHITE);

        // 5. Center Circular OK Button (Blank circle per user request)
        let r_ok = 24.0;
        let rect_ok = Rect::from_center_size(c, vec2(r_ok * 2.0, r_ok * 2.0));
        let resp_ok = ui.interact(rect_ok, ui.scope_id().with("dpad_ok"), Sense::click());
        if resp_ok.clicked() {
            self.ctrl.key(keys::DPAD_CENTER);
        }
        let bg_ok = if resp_ok.is_pointer_button_down_on() {
            ACCENT_ACTIVE
        } else if resp_ok.hovered() {
            ACCENT_HOVER
        } else {
            ACCENT
        };
        painter.circle_filled(c, r_ok, bg_ok);
        painter.circle_stroke(c, r_ok, Stroke::new(1.2, Color32::from_rgb(80, 160, 255)));
    }

    // -----------------------------------------------------------------------
    // Navigation Row (Back, Home, Menu)
    // -----------------------------------------------------------------------

    fn render_nav_row(&mut self, ui: &mut Ui, w: f32) {
        let gap = 8.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 36.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Back", BtnIcon::Back, None, 12.0, false) {
                self.ctrl.key(keys::BACK);
            }
            if custom_button(ui, vec2(btn_w, h), "Home", BtnIcon::Home, None, 12.0, false) {
                self.ctrl.key(keys::HOME);
            }
            if custom_button(ui, vec2(btn_w, h), "Menu", BtnIcon::Menu, None, 12.0, false) {
                self.ctrl.key(keys::MENU);
            }
        });
    }

    // -----------------------------------------------------------------------
    // Function Rows
    // -----------------------------------------------------------------------

    fn render_function_row_1(&mut self, ui: &mut Ui, w: f32) {
        let gap = 8.0;
        let btn_w = (w - 3.0 * gap) / 4.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Vol -", BtnIcon::VolMinus, None, 10.0, true) {
                self.ctrl.key(keys::VOLUME_DOWN);
            }
            if custom_button(ui, vec2(btn_w, h), "Mute", BtnIcon::Mute, None, 10.0, false) {
                self.ctrl.key(keys::VOLUME_MUTE);
            }
            if custom_button(ui, vec2(btn_w, h), "Vol +", BtnIcon::VolPlus, None, 10.0, true) {
                self.ctrl.key(keys::VOLUME_UP);
            }
            if custom_button(ui, vec2(btn_w, h), "Input", BtnIcon::Input, None, 10.0, false) {
                self.ctrl.key(keys::TV_INPUT);
            }
        });
    }

    fn render_function_row_2(&mut self, ui: &mut Ui, w: f32) {
        let gap = 8.0;
        let btn_w = (w - 3.0 * gap) / 4.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Ch -", BtnIcon::ChevronDown, None, 10.0, true) {
                self.ctrl.key(keys::CHANNEL_DOWN);
            }
            if custom_button(ui, vec2(btn_w, h), "Ch +", BtnIcon::ChevronUp, None, 10.0, true) {
                self.ctrl.key(keys::CHANNEL_UP);
            }
            if custom_button(ui, vec2(btn_w, h), "Guide", BtnIcon::Guide, None, 10.0, false) {
                self.ctrl.key(keys::GUIDE);
            }
            if custom_button(ui, vec2(btn_w, h), "Info", BtnIcon::Info, None, 10.0, false) {
                self.ctrl.key(keys::INFO);
            }
        });
    }

    fn render_function_row_3(&mut self, ui: &mut Ui, w: f32) {
        let gap = 8.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "Prev", BtnIcon::Prev, None, 10.0, true) {
                self.ctrl.key(keys::MEDIA_PREVIOUS);
            }
            if custom_button(ui, vec2(btn_w, h), "Play/Pause", BtnIcon::PlayPause, None, 10.0, false) {
                self.ctrl.key(keys::MEDIA_PLAY_PAUSE);
            }
            if custom_button(ui, vec2(btn_w, h), "Next", BtnIcon::Next, None, 10.0, true) {
                self.ctrl.key(keys::MEDIA_NEXT);
            }
        });
    }

    fn render_function_row_4(&mut self, ui: &mut Ui, w: f32) {
        let gap = 8.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 34.0;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            // Icon-only buttons for Search, Captions, Settings
            if custom_button(ui, vec2(btn_w, h), "", BtnIcon::Search, None, 10.0, false) {
                self.ctrl.key(keys::SEARCH);
            }
            if custom_button(ui, vec2(btn_w, h), "", BtnIcon::Captions, None, 10.0, false) {
                self.ctrl.key(keys::CAPTIONS);
            }
            if custom_button(ui, vec2(btn_w, h), "", BtnIcon::Settings, None, 10.0, false) {
                self.ctrl.key(keys::SETTINGS);
            }
        });
    }

    // -----------------------------------------------------------------------
    // Apps Section
    // -----------------------------------------------------------------------

    fn render_apps_section(&mut self, ui: &mut Ui, w: f32) {
        ui.label(RichText::new("Apps").size(13.0).strong().color(TEXT_MUTED));
        ui.add_space(3.0);

        let gap = 8.0;
        let btn_w = (w - 2.0 * gap) / 3.0;
        let h = 34.0;

        // Row 1: YouTube, Netflix, Prime Video (Prime is icon-only)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "YouTube", BtnIcon::YouTube, None, 10.0, false) {
                self.ctrl.launch("https://www.youtube.com");
            }
            if custom_button(ui, vec2(btn_w, h), "Netflix", BtnIcon::Netflix, None, 10.0, false) {
                self.ctrl.launch("https://www.netflix.com/title");
            }
            if custom_button(ui, vec2(btn_w, h), "", BtnIcon::Prime, None, 10.0, false) {
                self.ctrl.launch("https://app.primevideo.com");
            }
        });

        ui.add_space(4.0);

        // Row 2: Disney+ (icon-only), Hotstar, Spotify
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(btn_w, h), "", BtnIcon::Disney, None, 10.0, false) {
                self.ctrl.launch("https://www.disneyplus.com");
            }
            if custom_button(ui, vec2(btn_w, h), "Hotstar", BtnIcon::Hotstar, None, 10.0, false) {
                self.ctrl.launch("https://www.hotstar.com");
            }
            if custom_button(ui, vec2(btn_w, h), "Spotify", BtnIcon::Spotify, None, 10.0, false) {
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
        draw_nav_touchpad(painter, c + vec2(0.0, -18.0), ICON_SLATE);
        painter.text(
            c + vec2(0.0, 14.0),
            Align2::CENTER_CENTER,
            "Swipe to Navigate  •  Tap to Select",
            FontId::proportional(14.0),
            TEXT_MUTED,
        );

        // Gesture handling
        const STEP: f32 = 42.0;
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

        // Virtual Keyboard & Direct Text Entry
        ui.label(RichText::new("Type on TV").size(15.0).strong().color(TEXT_WHITE));
        ui.label(RichText::new("Type or paste search queries and text directly into the TV").color(TEXT_MUTED).size(11.5));
        ui.add_space(8.0);

        // Direct Text Input Bar
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let te = egui::TextEdit::singleline(&mut self.typed_text)
                .hint_text("Type text here...")
                .desired_width(w - 74.0);
            let te_resp = ui.add(te);
            let enter_hit = te_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui.button("Send").clicked() || enter_hit {
                if !self.typed_text.is_empty() {
                    self.ctrl.send_text(&self.typed_text);
                    self.typed_text.clear();
                }
            }
        });
        ui.add_space(8.0);

        let gap = 4.0;
        let kw = ((w - 9.0 * gap) / 10.0).floor();
        for row in ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"] {
            let width = row.len() as f32 * kw + (row.len() as f32 - 1.0) * gap;
            centered(ui, width, |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for c in row.chars() {
                    if custom_button(ui, vec2(kw, 42.0), &c.to_string(), BtnIcon::None, None, 8.0, false) {
                        self.ctrl.send_text(&c.to_string());
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
                    self.ctrl.send_text(&c.to_string());
                }
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(w * 0.40, 46.0), "Space", BtnIcon::None, None, 10.0, false) {
                self.ctrl.send_text(" ");
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

    let bg = if resp.is_pointer_button_down_on() {
        custom_fill.unwrap_or(BTN_ACTIVE)
    } else if resp.hovered() {
        custom_fill.unwrap_or(BTN_HOVER)
    } else {
        custom_fill.unwrap_or(BTN_BG)
    };

    let painter = ui.painter();
    painter.rect_filled(rect, corner_radius, bg);
    painter.rect_stroke(rect, corner_radius, Stroke::new(1.0, BORDER), StrokeKind::Inside);

    let c = rect.center();
    let text_color = if custom_fill.is_some() {
        TEXT_WHITE
    } else {
        Color32::from_rgb(226, 232, 240)
    };

    let font_id = FontId::proportional(12.0);

    if icon == BtnIcon::None {
        painter.text(c, Align2::CENTER_CENTER, label, font_id, text_color);
    } else if label.is_empty() {
        draw_btn_icon(painter, c, icon, text_color);
    } else {
        let icon_w = match icon {
            BtnIcon::Prime => 22.0,
            BtnIcon::None => 0.0,
            _ => 15.0,
        };
        let gap = 5.0;
        let text_w = (label.len() as f32) * 6.6;
        let total_w = icon_w + gap + text_w;
        let start_x = c.x - total_w * 0.5;

        let icon_c = pos2(start_x + icon_w * 0.5, c.y);
        draw_btn_icon(painter, icon_c, icon, text_color);

        let text_p = pos2(start_x + icon_w + gap, c.y);
        painter.text(text_p, Align2::LEFT_CENTER, label, font_id, text_color);
    }

    fire
}

fn draw_btn_icon(painter: &egui::Painter, c: Pos2, icon: BtnIcon, color: Color32) {
    match icon {
        BtnIcon::None => {}
        BtnIcon::Power => draw_power(painter, c, 7.5, RED_POWER),
        BtnIcon::Tv => draw_tv(painter, c, color),
        BtnIcon::ChevronDown => draw_chevron_down(painter, c, 4.5, color),
        BtnIcon::ChevronUp => draw_chevron_up(painter, c, 4.5, color),
        BtnIcon::Gear | BtnIcon::Settings => draw_gear(painter, c, 8.5, color),
        BtnIcon::Back => draw_back(painter, c, color),
        BtnIcon::Home => draw_home(painter, c, color),
        BtnIcon::Menu => draw_menu(painter, c, color),
        BtnIcon::VolMinus => draw_speaker_vol_down(painter, c, color),
        BtnIcon::VolPlus => draw_speaker_vol_up(painter, c, color),
        BtnIcon::Mute => draw_speaker_mute(painter, c, color),
        BtnIcon::Input => draw_input(painter, c, color),
        BtnIcon::Guide => draw_guide(painter, c, color),
        BtnIcon::Info => draw_info(painter, c, color),
        BtnIcon::Prev => draw_prev(painter, c, color),
        BtnIcon::PlayPause => draw_play_pause(painter, c, color),
        BtnIcon::Next => draw_next(painter, c, color),
        BtnIcon::Search => draw_search(painter, c, color),
        BtnIcon::Captions => draw_captions(painter, c, color),
        BtnIcon::YouTube => draw_youtube(painter, c),
        BtnIcon::Netflix => draw_netflix(painter, c),
        BtnIcon::Prime => draw_prime(painter, c),
        BtnIcon::Disney => draw_disney(painter, c),
        BtnIcon::Hotstar => draw_hotstar(painter, c),
        BtnIcon::Spotify => draw_spotify(painter, c),
    }
}

// ---------------------------------------------------------------------------
// D-Pad Widgets
// ---------------------------------------------------------------------------



// ---------------------------------------------------------------------------
// Bottom Navigation Bar
// ---------------------------------------------------------------------------

fn bottom_nav_bar(ui: &mut Ui, current_tab: &mut Tab, w: f32) {
    let bar_h = 52.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, bar_h), Sense::hover());
    let painter = ui.painter();

    // Dark bar background and top border line
    painter.rect_filled(rect, 0.0, Color32::from_rgb(9, 14, 24));
    painter.line_segment(
        [rect.min, pos2(rect.max.x, rect.min.y)],
        Stroke::new(1.0, Color32::from_rgb(20, 32, 50)),
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
            *current_tab = tab;
        }

        let is_active = *current_tab == tab;
        let color = if is_active { ACCENT } else { TEXT_MUTED };

        // Icon
        let icon_c = pos2(item_rect.center().x, item_rect.min.y + 15.0);
        match tab {
            Tab::Remote => draw_nav_remote(painter, icon_c, color),
            Tab::Apps => draw_nav_apps(painter, icon_c, color),
            Tab::Touchpad => draw_nav_touchpad(painter, icon_c, color),
            Tab::More => draw_nav_more(painter, icon_c, color),
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
            let line_c = pos2(item_rect.center().x, item_rect.max.y - 3.0);
            painter.rect_filled(
                Rect::from_center_size(line_c, vec2(28.0, 2.5)),
                1.25,
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
