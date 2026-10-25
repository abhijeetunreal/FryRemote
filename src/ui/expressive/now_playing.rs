//! Contextual Now Playing Bar & Media Transport for Material 3 Expressive UI.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::state::{ConnStatus, Shared};
use crate::ui::state::{ActiveSheet, UiState};
use crate::ui::theme::ThemePalette;
use eframe::egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

pub fn format_app_name(package: &str) -> String {
    let lower = package.to_lowercase();
    if lower.contains("youtube") {
        "YouTube".to_string()
    } else if lower.contains("netflix") {
        "Netflix".to_string()
    } else if lower.contains("amazonvideo") || lower.contains("prime") {
        "Prime Video".to_string()
    } else if lower.contains("disney") {
        "Disney+".to_string()
    } else if lower.contains("spotify") {
        "Spotify".to_string()
    } else if lower.contains("apple") || lower.contains("atve") {
        "Apple TV".to_string()
    } else if lower.contains("plex") {
        "Plex".to_string()
    } else if lower.contains("kodi") {
        "Kodi".to_string()
    } else if lower.contains("twitch") {
        "Twitch".to_string()
    } else if lower.contains("hulu") {
        "Hulu".to_string()
    } else if lower.contains("max") || lower.contains("hbo") {
        "Max".to_string()
    } else if lower.contains("launcher") || lower.contains("tvlauncher") {
        "Home Screen".to_string()
    } else {
        // Fallback: take last part of package or raw string
        let name = package.split('.').last().unwrap_or(package);
        let mut chars = name.chars();
        match chars.next() {
            None => "App".to_string(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

pub fn render_now_playing_bar(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    now: f64,
) {
    let avail_w = ui.available_width();
    let card_h = ui.available_height().min(58.0).max(42.0);

    let is_connected = matches!(snap.status, ConnStatus::Connected);
    let is_powered = state.tv_powered && snap.powered.unwrap_or(true);
    let is_offline = !is_connected || !is_powered;

    // Card background & outline based on connectivity
    let (card_rect, _) = ui.allocate_exact_size(vec2(avail_w, card_h), Sense::hover());
    let painter = ui.painter();

    let bg = if is_offline {
        Color32::from_rgba_unmultiplied(
            palette.surface_low.r(),
            palette.surface_low.g(),
            palette.surface_low.b(),
            120,
        )
    } else {
        palette.surface_low
    };

    let stroke = if is_offline {
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 12))
    } else {
        palette.outline()
    };

    painter.rect_filled(card_rect, card_h * 0.42, bg);
    painter.rect_stroke(card_rect, card_h * 0.42, stroke, StrokeKind::Inside);

    // Left Section: Clickable media info
    let right_transport_w = (card_h * 2.1).clamp(96.0, 126.0);
    let left_info_w = card_rect.width() - right_transport_w - 12.0;
    let left_rect = Rect::from_min_size(card_rect.min, vec2(left_info_w, card_h));
    let left_resp = ui.interact(left_rect, ui.scope_id().with("now_playing_left"), Sense::click());

    // App Icon Badge
    let badge_size = (card_h * 0.70).clamp(30.0, 40.0);
    let badge_rect = Rect::from_center_size(
        pos2(card_rect.min.x + 10.0 + badge_size * 0.5, card_rect.center().y),
        vec2(badge_size, badge_size),
    );

    let (badge_bg, badge_fg) = if is_offline {
        (palette.surface_high, palette.text_muted)
    } else if let Some(ref app) = snap.current_app {
        if app.to_lowercase().contains("launcher") {
            (palette.secondary_container, palette.on_secondary_container)
        } else {
            (palette.primary_container, palette.on_primary_container)
        }
    } else {
        (palette.secondary_container, palette.on_secondary_container)
    };

    painter.rect_filled(badge_rect, badge_size * 0.35, badge_bg);

    let icon_dim = badge_size * 0.55;
    let icon_rect = Rect::from_center_size(badge_rect.center(), vec2(icon_dim, icon_dim));
    if is_offline {
        svg_tv(ui, icon_rect, badge_fg);
    } else if let Some(ref app) = snap.current_app {
        if app.to_lowercase().contains("launcher") {
            svg_home(ui, icon_rect, badge_fg);
        } else {
            svg_movie(ui, icon_rect, badge_fg);
        }
    } else {
        svg_home(ui, icon_rect, badge_fg);
    }

    // Dynamic Title & Subtitle Labels
    let text_x = badge_rect.max.x + 10.0;
    let app_name_buf = snap.current_app.as_deref().map(format_app_name);

    let (top_label, bottom_label, top_color, bottom_color) = if !is_powered {
        ("STANDBY", "TV is Powered Off", palette.text_muted, Color32::from_rgba_unmultiplied(palette.text_muted.r(), palette.text_muted.g(), palette.text_muted.b(), 160))
    } else if !is_connected {
        ("OFFLINE", "Not Connected to TV", palette.text_muted, Color32::from_rgba_unmultiplied(palette.text_muted.r(), palette.text_muted.g(), palette.text_muted.b(), 160))
    } else if let Some(ref name) = app_name_buf {
        ("NOW PLAYING", name.as_str(), palette.primary, palette.text)
    } else {
        ("IDLE", "Not Playing • Home", palette.primary, palette.text)
    };

    painter.text(
        pos2(text_x, card_rect.center().y - card_h * 0.15),
        Align2::LEFT_CENTER,
        top_label,
        FontId::proportional((card_h * 0.20).clamp(9.5, 11.5)),
        top_color,
    );

    painter.text(
        pos2(text_x, card_rect.center().y + card_h * 0.15),
        Align2::LEFT_CENTER,
        bottom_label,
        FontId::proportional((card_h * 0.23).clamp(11.0, 13.0)),
        bottom_color,
    );

    if left_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        if !is_offline {
            state.toggle_sheet(ActiveSheet::MediaScrubber);
        } else {
            state.send_command_feedback("Connect to TV for media info", now);
        }
    }

    // Right Section: Media Transport (Rewind 10s, Play/Pause squircle, Forward 10s)
    let transport_rect = Rect::from_min_max(
        pos2(card_rect.max.x - right_transport_w - 6.0, card_rect.min.y),
        pos2(card_rect.max.x - 6.0, card_rect.max.y),
    );

    let pp_size = (card_h * 0.65).clamp(28.0, 38.0);
    let rew_size = (card_h * 0.58).clamp(24.0, 34.0);
    let icon_dim = (rew_size * 0.58).clamp(14.0, 20.0);
    let center_x = transport_rect.center().x;
    let step_x = (right_transport_w * 0.32).clamp(28.0, 40.0);

    // 1. Rewind 10s
    let rew_rect = Rect::from_center_size(pos2(center_x - step_x, transport_rect.center().y), vec2(rew_size, rew_size));
    let rew_resp = ui.interact(rew_rect, ui.scope_id().with("quick_rewind"), Sense::click());
    if rew_resp.hovered() && !is_offline {
        painter.circle_filled(rew_rect.center(), rew_size * 0.5, palette.surface_high);
    }
    let rew_color = if is_offline {
        Color32::from_rgba_unmultiplied(palette.text_muted.r(), palette.text_muted.g(), palette.text_muted.b(), 90)
    } else if rew_resp.hovered() {
        palette.text
    } else {
        palette.text_muted
    };
    svg_replay_10(ui, Rect::from_center_size(rew_rect.center(), vec2(icon_dim, icon_dim)), rew_color);

    if rew_resp.clicked() {
        if is_offline {
            haptics::trigger(HapticStyle::Light);
            state.send_command_feedback("Offline", now);
        } else {
            haptics::trigger(HapticStyle::Light);
            ctrl.key(keys::MEDIA_REWIND);
            state.send_command_feedback("Rewind 10s", now);
        }
    }

    // 2. Center Play/Pause Squircle
    let pp_rect = Rect::from_center_size(pos2(center_x, transport_rect.center().y), vec2(pp_size, pp_size));
    let pp_resp = ui.interact(pp_rect, ui.scope_id().with("quick_play_pause"), Sense::click());

    let pp_draw_rect = if pp_resp.is_pointer_button_down_on() && !is_offline {
        pp_rect.shrink(1.5)
    } else {
        pp_rect
    };

    let pp_bg = if is_offline {
        Color32::from_rgba_unmultiplied(palette.surface_high.r(), palette.surface_high.g(), palette.surface_high.b(), 100)
    } else if pp_resp.hovered() {
        palette.primary
    } else {
        palette.primary
    };

    let pp_fg = if is_offline {
        Color32::from_rgba_unmultiplied(palette.text_muted.r(), palette.text_muted.g(), palette.text_muted.b(), 120)
    } else {
        palette.on_primary
    };

    painter.rect_filled(pp_draw_rect, pp_size * 0.38, pp_bg);

    let pp_icon_c = pp_draw_rect.center();
    if state.is_playing && !is_offline {
        // Pause bars
        let bar_w = (pp_size * 0.09).clamp(2.5, 3.5);
        let bar_h = (pp_size * 0.35).clamp(10.0, 14.0);
        painter.rect_filled(Rect::from_center_size(pp_icon_c + vec2(-bar_w * 1.2, 0.0), vec2(bar_w, bar_h)), 1.2, pp_fg);
        painter.rect_filled(Rect::from_center_size(pp_icon_c + vec2(bar_w * 1.2, 0.0), vec2(bar_w, bar_h)), 1.2, pp_fg);
    } else {
        // Play triangle
        let tri_w = (pp_size * 0.28).clamp(8.0, 11.0);
        let tri_h = (pp_size * 0.36).clamp(10.0, 14.0);
        let pts = vec![pp_icon_c + vec2(-tri_w * 0.4, -tri_h * 0.5), pp_icon_c + vec2(tri_w * 0.6, 0.0), pp_icon_c + vec2(-tri_w * 0.4, tri_h * 0.5)];
        painter.add(eframe::egui::Shape::convex_polygon(pts, pp_fg, eframe::egui::Stroke::NONE));
    }

    if pp_resp.clicked() {
        if is_offline {
            haptics::trigger(HapticStyle::Light);
            state.send_command_feedback("Offline", now);
        } else {
            haptics::trigger(HapticStyle::Medium);
            state.is_playing = !state.is_playing;
            ctrl.key(keys::MEDIA_PLAY_PAUSE);
            let msg = if state.is_playing { "Playing" } else { "Paused" };
            state.send_command_feedback(msg, now);
        }
    }

    // 3. Forward 10s
    let fwd_rect = Rect::from_center_size(pos2(center_x + step_x, transport_rect.center().y), vec2(rew_size, rew_size));
    let fwd_resp = ui.interact(fwd_rect, ui.scope_id().with("quick_forward"), Sense::click());
    if fwd_resp.hovered() && !is_offline {
        painter.circle_filled(fwd_rect.center(), rew_size * 0.5, palette.surface_high);
    }
    let fwd_color = if is_offline {
        Color32::from_rgba_unmultiplied(palette.text_muted.r(), palette.text_muted.g(), palette.text_muted.b(), 90)
    } else if fwd_resp.hovered() {
        palette.text
    } else {
        palette.text_muted
    };
    svg_forward_10(ui, Rect::from_center_size(fwd_rect.center(), vec2(icon_dim, icon_dim)), fwd_color);

    if fwd_resp.clicked() {
        if is_offline {
            haptics::trigger(HapticStyle::Light);
            state.send_command_feedback("Offline", now);
        } else {
            haptics::trigger(HapticStyle::Light);
            ctrl.key(keys::MEDIA_FAST_FORWARD);
            state.send_command_feedback("Forward 10s", now);
        }
    }
}
