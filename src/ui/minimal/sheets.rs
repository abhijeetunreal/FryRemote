//! Overlay Sheets & Modals for Minimal Apple TV Edition.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::state::{ConnStatus, Shared, Tv};
use crate::ui::state::{ActiveSheet, UiState};
use crate::ui::theme::ThemePalette;
use eframe::egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

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

    let sheet_w = total_rect.width().min(330.0);
    let sheet_x = (total_rect.width() - sheet_w) * 0.5;

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
    let sheet_h = 280.0;
    let sheet_rect = Rect::from_min_size(pos2(x, bottom_y - sheet_h), vec2(w, sheet_h));
    let painter = ui.painter();

    painter.rect_filled(sheet_rect, 32.0, palette.surface);
    painter.rect_stroke(sheet_rect, 32.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

    // Grab handle
    let handle_rect = Rect::from_center_size(pos2(sheet_rect.center().x, sheet_rect.min.y + 12.0), vec2(36.0, 4.0));
    painter.rect_filled(handle_rect, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 60));

    let title_y = sheet_rect.min.y + 28.0;
    painter.text(
        pos2(sheet_rect.center().x, title_y),
        Align2::CENTER_CENTER,
        "Connected Devices",
        FontId::proportional(13.0),
        palette.text,
    );

    let tv_list_top = sheet_rect.min.y + 48.0;
    let tvs = if !ctrl.store.tvs.is_empty() {
        ctrl.store.tvs.clone()
    } else {
        vec![Tv {
            name: "Living Room TV".to_string(),
            host: "192.168.1.50".to_string(),
            paired: true,
        }]
    };

    let item_h = 44.0;
    for (i, tv) in tvs.iter().enumerate() {
        let ty = tv_list_top + (i as f32) * (item_h + 6.0);
        let item_rect = Rect::from_min_size(pos2(sheet_rect.min.x + 14.0, ty), vec2(w - 28.0, item_h));
        let is_connected = matches!(snap.status, ConnStatus::Connected);
        let resp = ui.interact(item_rect, ui.scope_id().with(&tv.host), Sense::click());

        let bg = if resp.hovered() { palette.surface_highest } else { palette.surface_high };
        painter.rect_filled(item_rect, 14.0, bg);
        painter.rect_stroke(item_rect, 14.0, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

        // LED dot
        let dot_color = if is_connected { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(251, 191, 36) };
        painter.circle_filled(pos2(item_rect.min.x + 16.0, item_rect.center().y), 4.0, dot_color);

        painter.text(
            pos2(item_rect.min.x + 30.0, item_rect.center().y),
            Align2::LEFT_CENTER,
            &tv.name,
            FontId::proportional(12.5),
            palette.text,
        );

        if resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            ctrl.connect(tv);
            state.send_command_feedback(format!("Connecting to {}", tv.name), now);
            state.active_sheet = ActiveSheet::None;
        }
    }

    // Add TV button
    let add_rect = Rect::from_min_size(pos2(sheet_rect.min.x + 14.0, sheet_rect.max.y - 50.0), vec2(w - 28.0, 38.0));
    let add_resp = ui.interact(add_rect, ui.scope_id().with("add_tv_btn"), Sense::click());
    let add_bg = if add_resp.hovered() { palette.primary } else { palette.surface_high };
    painter.rect_filled(add_rect, 14.0, add_bg);
    let add_fg = if add_resp.hovered() { palette.on_primary } else { palette.primary };
    painter.text(
        add_rect.center(),
        Align2::CENTER_CENTER,
        "+ Pair New TV by IP",
        FontId::proportional(12.0),
        add_fg,
    );

    if add_resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        state.active_sheet = ActiveSheet::None;
        let new_tv = Tv {
            name: "Android TV".to_string(),
            host: "192.168.1.100".to_string(),
            paired: false,
        };
        ctrl.start_pairing(&new_tv);
        *screen = crate::app::Screen::Pair(new_tv);
    }
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
