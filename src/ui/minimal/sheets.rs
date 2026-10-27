//! Overlay Sheets & Modals for Minimal Apple TV Edition.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::state::{Shared, Tv};
use crate::ui::common::widgets::keypad;
use crate::ui::state::{ActiveSheet, UiState};
use crate::ui::theme::ThemePalette;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

pub fn render_minimal_sheets(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    screen: &mut crate::app::Screen,
    now: f64,
) {
    if state.active_sheet == ActiveSheet::None {
        return;
    }

    let total_rect = ui.max_rect();

    // Dark translucent backdrop
    let backdrop_resp = ui.interact(total_rect, ui.scope_id().with("sheet_backdrop"), Sense::click());
    ui.painter().rect_filled(
        total_rect,
        0.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 160),
    );

    if backdrop_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.active_sheet = ActiveSheet::None;
        return;
    }

    let total_w = total_rect.width();
    let is_desktop_wide = total_w > 480.0;
    let sheet_w = if is_desktop_wide {
        (total_w * 0.40).clamp(360.0, 460.0)
    } else {
        (total_w - 20.0).clamp(280.0, 420.0)
    };
    let sheet_x = (total_w - sheet_w) * 0.5;

    match state.active_sheet {
        ActiveSheet::Apps => {
            render_apps_sheet(ui, palette, state, ctrl, sheet_x, sheet_w, total_rect.max.y, now);
        }
        ActiveSheet::Device => {
            render_device_sheet(ui, palette, state, ctrl, snap, sheet_x, sheet_w, total_rect.max.y, screen, now);
        }
        ActiveSheet::Voice => {
            render_voice_sheet(ui, palette, state, total_rect, now);
        }
        _ => {
            state.active_sheet = ActiveSheet::None;
        }
    }
}

fn render_apps_sheet(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    x: f32,
    w: f32,
    bottom_y: f32,
    now: f64,
) {
    let sheet_h = 240.0;
    let sheet_rect = Rect::from_min_size(pos2(x, bottom_y - sheet_h), vec2(w, sheet_h));
    let painter = ui.painter();

    painter.rect_filled(sheet_rect, 32.0, palette.surface);
    painter.rect_stroke(sheet_rect, 32.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

    // Grab handle
    let handle_w = 36.0;
    let handle_rect = Rect::from_center_size(pos2(sheet_rect.center().x, sheet_rect.min.y + 12.0), vec2(handle_w, 4.0));
    painter.rect_filled(handle_rect, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60));

    let title_y = sheet_rect.min.y + 28.0;
    painter.text(
        pos2(sheet_rect.center().x, title_y),
        Align2::CENTER_CENTER,
        "TV Apps",
        FontId::proportional(13.0),
        palette.text,
    );

    let apps = [
        ("Netflix", "https://www.netflix.com"),
        ("YouTube", "https://www.youtube.com"),
        ("Apple TV+", "https://tv.apple.com"),
        ("Prime Video", "https://www.amazon.com/primevideo"),
        ("Disney+", "https://www.disneyplus.com"),
        ("Spotify", "https://open.spotify.com"),
    ];

    let grid_top = sheet_rect.min.y + 48.0;
    let col_w = (w - 36.0) / 3.0;
    let btn_h = 68.0;

    for (i, &(name, url)) in apps.iter().enumerate() {
        let col = i % 3;
        let row = i / 3;
        let bx = sheet_rect.min.x + 12.0 + (col as f32) * (col_w + 6.0);
        let by = grid_top + (row as f32) * (btn_h + 8.0);
        let b_rect = Rect::from_min_size(pos2(bx, by), vec2(col_w, btn_h));
        let resp = ui.interact(b_rect, ui.scope_id().with(name), Sense::click());

        let bg = if resp.is_pointer_button_down_on() {
            palette.surface_highest
        } else if resp.hovered() {
            palette.surface_high
        } else {
            palette.surface_high
        };

        painter.rect_filled(b_rect, 16.0, bg);
        painter.rect_stroke(b_rect, 16.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

        // App Icon
        let icon_box = Rect::from_center_size(pos2(b_rect.center().x, b_rect.center().y - 8.0), vec2(22.0, 22.0));
        match name {
            "Netflix" => svg_netflix(ui, icon_box),
            "YouTube" => svg_youtube(ui, icon_box),
            "Apple TV+" => svg_tv(ui, icon_box, palette.text),
            "Prime Video" => svg_prime(ui, icon_box),
            "Disney+" => svg_disney(ui, icon_box),
            "Spotify" => svg_spotify(ui, icon_box),
            _ => svg_movie(ui, icon_box, palette.primary),
        }

        painter.text(
            pos2(b_rect.center().x, b_rect.center().y + 16.0),
            Align2::CENTER_CENTER,
            name,
            FontId::proportional(10.5),
            palette.text,
        );

        if resp.clicked() {
            haptics::trigger(HapticStyle::Medium);
            ctrl.launch(url);
            state.active_app_name = name.to_string();
            state.send_command_feedback(format!("Launch {name}"), now);
            state.active_sheet = ActiveSheet::None;
        }
    }
}

fn render_device_sheet(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    x: f32,
    w: f32,
    bottom_y: f32,
    screen: &mut crate::app::Screen,
    now: f64,
) {
    let base_h: f32 = if state.show_ip_pad { 470.0 } else { 390.0 };
    let sheet_h = base_h.min(bottom_y - 30.0);
    let sheet_rect = Rect::from_min_size(pos2(x, bottom_y - sheet_h), vec2(w, sheet_h));
    let painter = ui.painter();

    painter.rect_filled(sheet_rect, 32.0, palette.surface);
    painter.rect_stroke(sheet_rect, 32.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

    // Grab handle
    let handle_rect = Rect::from_center_size(pos2(sheet_rect.center().x, sheet_rect.min.y + 12.0), vec2(36.0, 4.0));
    painter.rect_filled(handle_rect, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60));

    // Title & Scan Button row
    let title_y = sheet_rect.min.y + 26.0;
    painter.text(
        pos2(sheet_rect.min.x + 20.0, title_y),
        Align2::LEFT_CENTER,
        "Google & Android TV",
        FontId::proportional(14.0),
        palette.text,
    );

    // Scan button on top right
    let scan_w = 70.0;
    let scan_h = 24.0;
    let scan_rect = Rect::from_min_size(
        pos2(sheet_rect.max.x - 20.0 - scan_w, title_y - 12.0),
        vec2(scan_w, scan_h),
    );
    let scan_resp = ui.interact(scan_rect, ui.scope_id().with("minimal_scan_btn"), Sense::click());
    let scan_bg = if snap.scanning {
        palette.surface_high
    } else if scan_resp.hovered() {
        palette.primary
    } else {
        palette.surface_high
    };
    let scan_fg = if scan_resp.hovered() && !snap.scanning {
        palette.on_primary
    } else {
        palette.primary
    };
    painter.rect_filled(scan_rect, 12.0, scan_bg);
    painter.rect_stroke(scan_rect, 12.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);
    painter.text(
        scan_rect.center(),
        Align2::CENTER_CENTER,
        if snap.scanning { "Scanning…" } else { "⟳ Scan" },
        FontId::proportional(11.0),
        scan_fg,
    );
    if scan_resp.clicked() && !snap.scanning {
        haptics::trigger(HapticStyle::Light);
        ctrl.scan();
        state.send_command_feedback("Scanning Wi-Fi...".to_string(), now);
    }

    // Scrollable device content
    let content_rect = Rect::from_min_max(
        pos2(sheet_rect.min.x + 14.0, sheet_rect.min.y + 44.0),
        pos2(sheet_rect.max.x - 14.0, sheet_rect.max.y - 10.0),
    );

    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child_ui, |ui| {
            ui.add_space(4.0);

            // 1. Saved TVs
            if !ctrl.store.tvs.is_empty() {
                ui.label(egui::RichText::new("Saved TVs").strong().size(12.0).color(palette.text_muted));
                ui.add_space(4.0);

                for tv in ctrl.store.tvs.clone() {
                    let is_active = ctrl.active.as_ref().map(|t| t.host == tv.host).unwrap_or(false);
                    let (card_rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
                    let p = ui.painter();

                    let bg = if is_active {
                        palette.primary_container
                    } else if resp.hovered() {
                        palette.surface_highest
                    } else {
                        palette.surface_high
                    };
                    p.rect_filled(card_rect, 12.0, bg);
                    p.rect_stroke(card_rect, 12.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

                    let dot_c = pos2(card_rect.min.x + 16.0, card_rect.center().y);
                    let dot_col = if is_active { Color32::from_rgb(52, 211, 153) } else { palette.text_muted };
                    p.circle_filled(dot_c, 3.5, dot_col);

                    p.text(
                        pos2(card_rect.min.x + 30.0, card_rect.center().y - 6.0),
                        Align2::LEFT_CENTER,
                        &tv.name,
                        FontId::proportional(12.5),
                        if is_active { palette.on_primary_container } else { palette.text },
                    );

                    let sub = format!("{} • {}", tv.host, if is_active { "Connected" } else { "Saved" });
                    p.text(
                        pos2(card_rect.min.x + 30.0, card_rect.center().y + 8.0),
                        Align2::LEFT_CENTER,
                        sub,
                        FontId::proportional(10.0),
                        if is_active { palette.on_primary_container } else { palette.text_muted },
                    );

                    if resp.clicked() {
                        haptics::trigger(HapticStyle::Light);
                        if !is_active {
                            ctrl.connect(&tv);
                            state.send_command_feedback(format!("Connecting to {}", tv.name), now);
                        }
                        state.active_sheet = ActiveSheet::None;
                    }

                    ui.add_space(4.0);
                }
                ui.add_space(8.0);
            }

            // 2. Nearby Discovered TVs
            ui.label(egui::RichText::new("Nearby on Wi-Fi").strong().size(12.0).color(palette.text_muted));
            ui.add_space(4.0);

            if snap.scanning {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(egui::RichText::new("Scanning for nearby Google & Android TVs…").size(12.0).color(palette.text));
                });
                ui.add_space(6.0);
            }

            let unadded: Vec<_> = snap.discovered.iter().filter(|d| !ctrl.store.tvs.iter().any(|t| t.host == d.host)).cloned().collect();

            if !unadded.is_empty() {
                for tv in unadded {
                    let (card_rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
                    let p = ui.painter();

                    let bg = if resp.hovered() { palette.surface_highest } else { palette.surface_high };
                    p.rect_filled(card_rect, 12.0, bg);
                    p.rect_stroke(card_rect, 12.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

                    let icon_box = Rect::from_center_size(pos2(card_rect.min.x + 20.0, card_rect.center().y), vec2(18.0, 18.0));
                    svg_tv(ui, icon_box, palette.primary);

                    p.text(
                        pos2(card_rect.min.x + 38.0, card_rect.center().y - 6.0),
                        Align2::LEFT_CENTER,
                        &tv.name,
                        FontId::proportional(12.5),
                        palette.text,
                    );
                    p.text(
                        pos2(card_rect.min.x + 38.0, card_rect.center().y + 8.0),
                        Align2::LEFT_CENTER,
                        format!("{} • Tap to Pair", tv.host),
                        FontId::proportional(10.0),
                        palette.primary,
                    );

                    if resp.clicked() {
                        haptics::trigger(HapticStyle::Medium);
                        state.active_sheet = ActiveSheet::None;
                        ctrl.store.upsert(tv.clone());
                        ctrl.start_pairing(&tv);
                        *screen = crate::app::Screen::Pair(tv);
                    }
                    ui.add_space(4.0);
                }
            } else if !snap.scanning {
                let (empty_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 76.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(empty_rect, 12.0, palette.surface_low);
                p.rect_stroke(empty_rect, 12.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

                p.text(
                    pos2(empty_rect.center().x, empty_rect.min.y + 20.0),
                    Align2::CENTER_CENTER,
                    "No TVs Available Nearby",
                    FontId::proportional(12.5),
                    palette.text,
                );
                p.text(
                    pos2(empty_rect.center().x, empty_rect.min.y + 40.0),
                    Align2::CENTER_CENTER,
                    "Ensure your TV is awake & on the same Wi-Fi network",
                    FontId::proportional(10.5),
                    palette.text_muted,
                );

                // Scan Again inline button
                let rescan_btn = Rect::from_center_size(pos2(empty_rect.center().x, empty_rect.min.y + 60.0), vec2(90.0, 20.0));
                let rescan_resp = ui.interact(rescan_btn, ui.scope_id().with("rescan_btn_inner"), Sense::click());
                if rescan_resp.hovered() {
                    p.rect_filled(rescan_btn, 6.0, palette.primary);
                    p.text(rescan_btn.center(), Align2::CENTER_CENTER, "Scan Again", FontId::proportional(10.0), palette.on_primary);
                } else {
                    p.rect_filled(rescan_btn, 6.0, palette.surface_high);
                    p.text(rescan_btn.center(), Align2::CENTER_CENTER, "Scan Again", FontId::proportional(10.0), palette.primary);
                }
                if rescan_resp.clicked() {
                    haptics::trigger(HapticStyle::Light);
                    ctrl.scan();
                    state.send_command_feedback("Scanning Wi-Fi...".to_string(), now);
                }

                ui.add_space(6.0);
            }

            ui.add_space(8.0);

            // 3. Add TV by IP Address
            let ip_btn_text = if state.show_ip_pad { "Hide IP Entry" } else { "+ Add TV by IP Address" };
            if ui.add(egui::Button::new(egui::RichText::new(ip_btn_text).size(12.0).color(palette.primary)).fill(palette.surface_high).corner_radius(12.0).min_size(vec2(ui.available_width(), 36.0))).clicked() {
                haptics::trigger(HapticStyle::Light);
                state.show_ip_pad = !state.show_ip_pad;
            }

            if state.show_ip_pad {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(if state.ip_input.is_empty() { "192.168.1._" } else { &state.ip_input }).monospace().size(15.0).color(palette.primary));
                keypad(ui, &mut state.ip_input, false, 15);
                let valid = state.ip_input.parse::<std::net::Ipv4Addr>().is_ok();
                if ui.add_enabled(valid, egui::Button::new(egui::RichText::new("Pair & Connect").color(palette.on_primary)).fill(palette.primary).corner_radius(12.0).min_size(vec2(ui.available_width(), 36.0))).clicked() {
                    haptics::trigger(HapticStyle::Medium);
                    let host = state.ip_input.clone();
                    let new_tv = Tv { name: format!("Google TV ({host})"), host, paired: false };
                    ctrl.store.upsert(new_tv.clone());
                    ctrl.start_pairing(&new_tv);
                    *screen = crate::app::Screen::Pair(new_tv);
                    state.ip_input.clear();
                    state.show_ip_pad = false;
                    state.active_sheet = ActiveSheet::None;
                }
            }

            ui.add_space(10.0);
        });
}

fn render_voice_sheet(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    rect: Rect,
    now: f64,
) {
    let painter = ui.painter();
    let c = rect.center();

    // Pulse animation
    let pulse = ((now * 3.0).sin() * 0.5 + 0.5) as f32;
    let r_outer = 48.0 + pulse * 18.0;
    let alpha = ((1.0 - pulse) * 120.0) as u8;
    painter.circle_filled(
        c + vec2(0.0, -20.0),
        r_outer,
        Color32::from_rgba_unmultiplied(palette.primary.r(), palette.primary.g(), palette.primary.b(), alpha),
    );

    // Center mic container
    let mic_r = 34.0;
    painter.circle_filled(c + vec2(0.0, -20.0), mic_r, palette.primary_container);

    let mic_rect = Rect::from_center_size(c + vec2(0.0, -20.0), vec2(28.0, 28.0));
    svg_mic(ui, mic_rect, palette.on_primary_container);

    painter.text(
        c + vec2(0.0, 36.0),
        Align2::CENTER_CENTER,
        "Listening...",
        FontId::proportional(15.0),
        palette.text,
    );

    painter.text(
        c + vec2(0.0, 58.0),
        Align2::CENTER_CENTER,
        "Tap anywhere to dismiss",
        FontId::proportional(11.5),
        palette.text_muted,
    );

    let resp = ui.interact(rect, ui.scope_id().with("voice_dismiss"), Sense::click());
    if resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.active_sheet = ActiveSheet::None;
    }
}
