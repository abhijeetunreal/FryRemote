//! Advanced Mode (Full-Screen Responsive Layout) for FryRemote.

pub mod apps_tab;
pub mod more_tab;
pub mod remote_tab;
pub mod touchpad_tab;

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::state::{ConnStatus, Shared, Tv};
use crate::ui::common::widgets::keypad;
use crate::ui::state::{AdvancedTab, UiState};
use crate::ui::UiMode;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui};

pub fn render_advanced_ui(
    ui: &mut Ui,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    ui_mode: &mut UiMode,
    screen: &mut crate::app::Screen,
) {
    let now = ui.input(|i| i.time);
    let total_rect = ui.max_rect();
    let painter = ui.painter();

    // 1. Full-screen slate background & ambient wave
    painter.rect_filled(total_rect, 0.0, Color32::from_rgb(9, 13, 22));
    draw_ambient_background(painter, total_rect);

    // Responsive margins: utilizes full screen
    let margin_x = if total_rect.width() > 500.0 { 24.0 } else { 14.0 };
    let content_rect = total_rect.shrink2(vec2(margin_x, 6.0));

    ui.allocate_ui(content_rect.size(), |ui| {
        let w = ui.available_width();

        // Status bar
        draw_status_bar(ui, w);
        ui.add_space(4.0);

        // Header: Logo, FryRemote title, Mode Switcher button, Settings Gear
        render_header(ui, state, ctrl, snap, w, ui_mode);
        ui.add_space(8.0);

        // TV Card ("Living Room / Google TV" + "● Connected" status pill)
        render_tv_card(ui, state, ctrl, snap, w);
        ui.add_space(8.0);

        // TV Selector drawer (when open)
        if state.show_tv_selector {
            render_tv_selector_drawer(ui, state, ctrl, snap, w, screen);
            ui.add_space(8.0);
        }

        // Viewport height
        let nav_h = 52.0;
        let viewport_h = (ui.available_height() - nav_h).max(150.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(viewport_h)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                match state.advanced_tab {
                    AdvancedTab::Remote => {
                        remote_tab::render_advanced_remote_tab(ui, state, ctrl, snap, w, now);
                    }
                    AdvancedTab::Apps => {
                        apps_tab::render_advanced_apps_tab(ui, ctrl, w);
                    }
                    AdvancedTab::Touchpad => {
                        touchpad_tab::render_advanced_touchpad_tab(ui, state, ctrl, w, viewport_h);
                    }
                    AdvancedTab::More => {
                        more_tab::render_advanced_more_tab(ui, state, ctrl, snap, w, screen);
                    }
                }
            });

        // Fixed bottom navigation bar (Remote, Apps, Touchpad, More)
        bottom_nav_bar(ui, state, w);
    });
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

fn render_header(
    ui: &mut Ui,
    state: &mut UiState,
    _ctrl: &mut Controller,
    _snap: &Shared,
    _w: f32,
    ui_mode: &mut UiMode,
) {
    ui.horizontal(|ui| {
        // Brand Logo
        let (logo_rect, _) = ui.allocate_exact_size(vec2(28.0, 36.0), Sense::hover());
        svg_brand_logo(ui, logo_rect);

        ui.add_space(8.0);

        // Title
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label(RichText::new("Fry").size(22.0).strong().color(Color32::from_rgb(248, 250, 252)));
                ui.label(RichText::new("Remote").size(22.0).strong().color(Color32::from_rgb(59, 130, 246)));
            });
            ui.label(RichText::new("Advanced Mode • Full Screen").size(11.0).color(Color32::from_rgb(100, 116, 139)));
        });

        // Right side: Mode Switcher + Settings Gear
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Gear button
            let (gear_rect, gear_resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());
            let gear_color = if gear_resp.hovered() { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) };
            svg_settings(ui, Rect::from_center_size(gear_rect.center(), vec2(19.0, 19.0)), gear_color);
            if gear_resp.clicked() {
                haptics::trigger(HapticStyle::Light);
                state.show_tv_selector = !state.show_tv_selector;
            }

            // Mode switcher pill
            let mode_btn = egui::Button::new(RichText::new("✨ Expressive UI").size(11.5).strong().color(Color32::from_rgb(147, 197, 253)))
                .fill(Color32::from_rgb(20, 35, 60))
                .stroke(Stroke::new(1.0, Color32::from_rgb(59, 130, 246)))
                .corner_radius(14.0);
            if ui.add(mode_btn).clicked() {
                haptics::trigger(HapticStyle::Heavy);
                *ui_mode = UiMode::Expressive;
            }
        });
    });
}

fn render_tv_card(
    ui: &mut Ui,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    w: f32,
) {
    let card_size = vec2(w, 50.0);
    let (rect, resp) = ui.allocate_exact_size(card_size, Sense::click());
    let bg = if resp.is_pointer_button_down_on() {
        Color32::from_rgb(36, 56, 92)
    } else if resp.hovered() {
        Color32::from_rgb(20, 31, 50)
    } else {
        Color32::from_rgb(15, 23, 42)
    };

    let draw_card_rect = if resp.is_pointer_button_down_on() { rect.shrink(0.8) } else { rect };
    let painter = ui.painter();
    painter.rect_filled(draw_card_rect, 14.0, bg);
    painter.rect_stroke(draw_card_rect, 14.0, Stroke::new(1.0, Color32::from_rgb(28, 43, 68)), StrokeKind::Inside);

    // TV outline icon
    let tv_rect = Rect::from_center_size(pos2(draw_card_rect.min.x + 28.0, draw_card_rect.center().y), vec2(22.0, 16.0));
    svg_tv(ui, tv_rect, Color32::from_rgb(59, 130, 246));

    let active_name = ctrl
        .active
        .as_ref()
        .map(|t| t.name.clone())
        .or_else(|| ctrl.store.tvs.first().map(|t| t.name.clone()))
        .unwrap_or_else(|| "Living Room".to_string());

    let text_x = draw_card_rect.min.x + 54.0;
    painter.text(
        pos2(text_x, draw_card_rect.center().y - 10.0),
        Align2::LEFT_CENTER,
        &active_name,
        FontId::proportional(15.5),
        Color32::WHITE,
    );
    painter.text(
        pos2(text_x, draw_card_rect.center().y + 10.0),
        Align2::LEFT_CENTER,
        "Google TV",
        FontId::proportional(12.0),
        Color32::from_rgb(100, 116, 139),
    );

    // Status pill
    let (pill_color, pill_text, bg_color, border_color) = match &snap.status {
        ConnStatus::Connected => (Color32::from_rgb(34, 197, 94), "Connected", Color32::from_rgb(11, 39, 32), Color32::from_rgb(18, 71, 56)),
        ConnStatus::Connecting => (Color32::from_rgb(255, 183, 77), "Connecting", Color32::from_rgb(38, 28, 12), Color32::from_rgb(70, 50, 20)),
        ConnStatus::Failed(_) => (Color32::from_rgb(255, 95, 95), "Offline", Color32::from_rgb(38, 14, 14), Color32::from_rgb(70, 25, 25)),
        ConnStatus::Idle => (Color32::from_rgb(34, 197, 94), "Connected", Color32::from_rgb(11, 39, 32), Color32::from_rgb(18, 71, 56)),
    };

    let pill_w = 90.0;
    let pill_h = 24.0;
    let pill_rect = Rect::from_center_size(pos2(draw_card_rect.max.x - 14.0 - pill_w * 0.5, draw_card_rect.center().y), vec2(pill_w, pill_h));
    painter.rect_filled(pill_rect, 12.0, bg_color);
    painter.rect_stroke(pill_rect, 12.0, Stroke::new(1.0, border_color), StrokeKind::Inside);

    painter.circle_filled(pos2(pill_rect.min.x + 13.0, pill_rect.center().y), 3.5, pill_color);
    painter.text(pos2(pill_rect.min.x + 23.0, pill_rect.center().y), Align2::LEFT_CENTER, pill_text, FontId::proportional(11.5), pill_color);

    if resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.show_tv_selector = !state.show_tv_selector;
    }
}

fn render_tv_selector_drawer(
    ui: &mut Ui,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    w: f32,
    screen: &mut crate::app::Screen,
) {
    egui::Frame::new()
        .fill(Color32::from_rgb(15, 23, 42))
        .corner_radius(16)
        .stroke(Stroke::new(1.0, Color32::from_rgb(28, 43, 68)))
        .inner_margin(14)
        .show(ui, |ui| {
            ui.set_width(w - 28.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Your Google TVs").size(16.0).strong().color(Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕ Close").clicked() {
                        state.show_tv_selector = false;
                    }
                });
            });
            ui.add_space(8.0);

            // Saved TVs
            for tv in ctrl.store.tvs.clone() {
                ui.horizontal(|ui| {
                    let is_active = ctrl.active.as_ref().map(|t| t.host == tv.host).unwrap_or(false);
                    let title = format!("{} ({})", tv.name, tv.host);
                    ui.label(RichText::new(title).color(if is_active { Color32::from_rgb(59, 130, 246) } else { Color32::WHITE }));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Forget").clicked() {
                            ctrl.store.remove(&tv.host);
                        }
                        if tv.paired {
                            if !is_active && ui.button("Connect").clicked() {
                                ctrl.connect(&tv);
                                state.show_tv_selector = false;
                            }
                        } else if ui.button("Pair").clicked() {
                            state.code_input.clear();
                            ctrl.start_pairing(&tv);
                            *screen = crate::app::Screen::Pair(tv.clone());
                            state.show_tv_selector = false;
                        }
                    });
                });
            }

            // Discovered TVs
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Discovered on Wi-Fi").strong().color(Color32::from_rgb(100, 116, 139)));
                if snap.scanning {
                    ui.spinner();
                } else if ui.button("Scan").clicked() {
                    ctrl.scan();
                }
            });

            for d in &snap.discovered {
                if !ctrl.store.tvs.iter().any(|t| t.host == d.host) {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} ({})", d.name, d.host));
                        if ui.button("Add & Pair").clicked() {
                            ctrl.store.upsert(d.clone());
                            ctrl.start_pairing(d);
                            *screen = crate::app::Screen::Pair(d.clone());
                            state.show_tv_selector = false;
                        }
                    });
                }
            }

            // Manual IP entry
            ui.add_space(6.0);
            if ui.button(if state.show_ip_pad { "Hide IP Entry" } else { "Add TV by IP Address" }).clicked() {
                state.show_ip_pad = !state.show_ip_pad;
            }
            if state.show_ip_pad {
                ui.label(RichText::new(if state.ip_input.is_empty() { "192.168.1._" } else { &state.ip_input }).monospace().size(18.0).color(Color32::from_rgb(59, 130, 246)));
                keypad(ui, &mut state.ip_input, false, 15);
                let valid = state.ip_input.parse::<std::net::Ipv4Addr>().is_ok();
                if ui.add_enabled(valid, egui::Button::new("Save & Connect").fill(Color32::from_rgb(59, 130, 246))).clicked() {
                    let host = state.ip_input.clone();
                    let new_tv = Tv { name: format!("Google TV ({host})"), host, paired: false };
                    ctrl.store.upsert(new_tv.clone());
                    ctrl.start_pairing(&new_tv);
                    *screen = crate::app::Screen::Pair(new_tv);
                    state.ip_input.clear();
                    state.show_ip_pad = false;
                    state.show_tv_selector = false;
                }
            }
        });
}

fn bottom_nav_bar(ui: &mut Ui, state: &mut UiState, w: f32) {
    let bar_h = 50.0;
    let (rect, _) = ui.allocate_exact_size(vec2(w, bar_h), Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(rect, 0.0, Color32::from_rgb(9, 13, 22));
    painter.line_segment(
        [rect.min, pos2(rect.max.x, rect.min.y)],
        Stroke::new(1.0, Color32::from_rgb(22, 34, 52)),
    );

    let items = [
        (AdvancedTab::Remote, "Remote"),
        (AdvancedTab::Apps, "Apps"),
        (AdvancedTab::Touchpad, "Touchpad"),
        (AdvancedTab::More, "More"),
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
            state.advanced_tab = tab;
        }

        let is_active = state.advanced_tab == tab;
        let color = if is_active { Color32::from_rgb(59, 130, 246) } else { Color32::from_rgb(100, 116, 139) };

        let icon_rect = Rect::from_center_size(pos2(item_rect.center().x, item_rect.min.y + 15.0), vec2(16.0, 16.0));
        match tab {
            AdvancedTab::Remote => svg_nav_remote(ui, icon_rect, color),
            AdvancedTab::Apps => svg_nav_apps(ui, icon_rect, color),
            AdvancedTab::Touchpad => svg_nav_touchpad(ui, icon_rect, color),
            AdvancedTab::More => svg_nav_more(ui, icon_rect, color),
        }

        painter.text(
            pos2(item_rect.center().x, item_rect.min.y + 33.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0),
            color,
        );

        if is_active {
            let line_c = pos2(item_rect.center().x, item_rect.max.y - 3.5);
            painter.rect_filled(Rect::from_center_size(line_c, vec2(24.0, 2.5)), 1.2, Color32::from_rgb(59, 130, 246));
        }
    }
}
