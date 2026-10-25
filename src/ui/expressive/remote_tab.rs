//! Tab 1: Remote Tab for Material 3 Expressive UI.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::state::Shared;
use crate::ui::common::dpad::{draw_expressive_dpad, draw_swipe_touchpad, DpadAction};
use crate::ui::common::widgets::M3Button;
use crate::ui::state::{ActiveSheet, PadMode, UiState};
use crate::ui::theme::ThemePalette;
use eframe::egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Shape, Stroke, StrokeKind, Ui};

pub fn render_expressive_remote_tab(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    vp_h: f32,
    now: f64,
) {
    let avail_w = ui.available_width();
    let effective_h = if vp_h.is_finite() && vp_h > 150.0 {
        vp_h
    } else {
        ui.available_height().min(900.0).max(400.0)
    };

    // Responsive scaling based on both available height and width
    let h_factor = (effective_h / 560.0).clamp(0.68, 1.35);
    let w_factor = (avail_w / 340.0).clamp(0.78, 1.35);
    let scale = h_factor * 0.65 + w_factor * 0.35;

    // Component sizes
    let pad_mode_h = (30.0 * scale).clamp(24.0, 34.0);
    let pad_mode_w = (165.0 * scale).clamp(136.0, 195.0);

    let dpad_size = (215.0 * scale)
        .clamp(160.0, 260.0)
        .min(avail_w - 20.0)
        .min(effective_h * 0.40);

    let nav_h = (48.0 * scale).clamp(38.0, 56.0);
    let nav_gap = (9.0 * scale).clamp(6.0, 12.0);
    let nav_btn_w = (avail_w - 2.0 * nav_gap) / 3.0;

    let rocker_h = (134.0 * scale).clamp(104.0, 170.0);
    let rocker_gap = nav_gap;
    let rocker_w = nav_btn_w;

    // Compute extra height and distribute
    let content_h = pad_mode_h + dpad_size + nav_h + rocker_h;
    let extra_h = (effective_h - content_h).max(0.0);

    let top_gap = (extra_h * 0.08).clamp(2.0, 10.0);
    let gap_pad = (extra_h * 0.16).clamp(4.0, 16.0);
    let gap_nav = (extra_h * 0.38).clamp(6.0, 24.0);
    let gap_rocker = (extra_h * 0.38).clamp(6.0, 24.0);

    ui.add_space(top_gap);

    // 1. Pad Mode Selector Pill (D-Pad vs Swipe)
    ui.horizontal(|ui| {
        let start_x = (avail_w - pad_mode_w) * 0.5;
        ui.add_space(start_x.max(0.0));

        let (pill_rect, _) = ui.allocate_exact_size(vec2(pad_mode_w, pad_mode_h), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(pill_rect, pad_mode_h * 0.5, palette.surface_low);
        painter.rect_stroke(pill_rect, pad_mode_h * 0.5, palette.outline(), StrokeKind::Inside);

        let half_w = pad_mode_w * 0.5;

        // D-pad mode button
        let dpad_rect = Rect::from_min_size(pill_rect.min, vec2(half_w, pad_mode_h));
        let dpad_resp = ui.interact(dpad_rect, ui.scope_id().with("mode_dpad"), Sense::click());
        let is_dpad = state.pad_mode == PadMode::Dpad;
        if is_dpad {
            painter.rect_filled(dpad_rect.shrink(2.0), (pad_mode_h - 4.0) * 0.5, palette.secondary_container);
        }
        let dpad_fg = if is_dpad { palette.on_secondary_container } else { palette.text_muted };
        let icon_dim = (pad_mode_h * 0.48).clamp(12.0, 16.0);
        let dpad_icon_rect = Rect::from_center_size(pos2(dpad_rect.min.x + pad_mode_h * 0.55, dpad_rect.center().y), vec2(icon_dim, icon_dim));
        svg_gamepad(ui, dpad_icon_rect, dpad_fg);
        painter.text(
            pos2(dpad_rect.min.x + pad_mode_h * 1.0, dpad_rect.center().y),
            Align2::LEFT_CENTER,
            "D-Pad",
            FontId::proportional((pad_mode_h * 0.38).clamp(10.5, 12.5)),
            dpad_fg,
        );
        if dpad_resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            state.pad_mode = PadMode::Dpad;
        }

        // Swipe mode button
        let swipe_rect = Rect::from_min_size(pos2(pill_rect.min.x + half_w, pill_rect.min.y), vec2(half_w, pad_mode_h));
        let swipe_resp = ui.interact(swipe_rect, ui.scope_id().with("mode_swipe"), Sense::click());
        let is_swipe = state.pad_mode == PadMode::Swipe;
        if is_swipe {
            painter.rect_filled(swipe_rect.shrink(2.0), (pad_mode_h - 4.0) * 0.5, palette.secondary_container);
        }
        let swipe_fg = if is_swipe { palette.on_secondary_container } else { palette.text_muted };
        let swipe_icon_rect = Rect::from_center_size(pos2(swipe_rect.min.x + pad_mode_h * 0.55, swipe_rect.center().y), vec2(icon_dim, icon_dim));
        svg_touch_app(ui, swipe_icon_rect, swipe_fg);
        painter.text(
            pos2(swipe_rect.min.x + pad_mode_h * 1.0, swipe_rect.center().y),
            Align2::LEFT_CENTER,
            "Swipe",
            FontId::proportional((pad_mode_h * 0.38).clamp(10.5, 12.5)),
            swipe_fg,
        );
        if swipe_resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            state.pad_mode = PadMode::Swipe;
        }
    });

    ui.add_space(gap_pad);

    // 2. Navigation Pad (D-Pad or Swipe)
    ui.horizontal(|ui| {
        let pad_x = (avail_w - dpad_size) * 0.5;
        ui.add_space(pad_x.max(0.0));

        let action = match state.pad_mode {
            PadMode::Dpad => draw_expressive_dpad(ui, palette, state, dpad_size),
            PadMode::Swipe => draw_swipe_touchpad(ui, palette, state, dpad_size, now),
        };

        if let Some(act) = action {
            match act {
                DpadAction::Up => {
                    ctrl.key(keys::DPAD_UP);
                    state.send_command_feedback("Up", now);
                }
                DpadAction::Down => {
                    ctrl.key(keys::DPAD_DOWN);
                    state.send_command_feedback("Down", now);
                }
                DpadAction::Left => {
                    ctrl.key(keys::DPAD_LEFT);
                    state.send_command_feedback("Left", now);
                }
                DpadAction::Right => {
                    ctrl.key(keys::DPAD_RIGHT);
                    state.send_command_feedback("Right", now);
                }
                DpadAction::Center => {
                    ctrl.key(keys::DPAD_CENTER);
                    state.send_command_feedback("Select", now);
                }
            }
        }
    });

    ui.add_space(gap_nav);

    // 3. Primary Navigation Row (Back • Home • Voice)
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = nav_gap;

        let nav_cr = nav_h * 0.5;

        // Back
        let (back_clicked, _) = M3Button::new(vec2(nav_btn_w, nav_h))
            .label("Back")
            .icon(crate::ui::common::widgets::BtnIcon::Back)
            .bg(palette.surface_high)
            .text_color(palette.text)
            .corner_radius(nav_cr)
            .show(ui, palette);
        if back_clicked {
            ctrl.key(keys::BACK);
            state.send_command_feedback("Back", now);
        }

        // Home (Prominent Tonal Pill)
        let (home_clicked, _) = M3Button::new(vec2(nav_btn_w, nav_h))
            .label("Home")
            .icon(crate::ui::common::widgets::BtnIcon::Home)
            .bg(palette.secondary_container)
            .text_color(palette.on_secondary_container)
            .corner_radius(nav_cr)
            .show(ui, palette);
        if home_clicked {
            ctrl.key(keys::HOME);
            state.send_command_feedback("Home", now);
        }

        // Voice Assistant (Expressive Tertiary Pill)
        let (voice_clicked, _) = M3Button::new(vec2(nav_btn_w, nav_h))
            .label("Voice")
            .icon(crate::ui::common::widgets::BtnIcon::Mic)
            .bg(palette.tertiary_container)
            .text_color(palette.tertiary)
            .corner_radius(nav_cr)
            .show(ui, palette);
        if voice_clicked {
            haptics::trigger(HapticStyle::Heavy);
            state.toggle_sheet(ActiveSheet::Voice);
        }
    });

    ui.add_space(gap_rocker);

    // 4. Ergonomic Vertical Rockers (Volume + Center Quick Actions + Channel)
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = rocker_gap;

        // LEFT: VOLUME VERTICAL ROCKER
        render_volume_rocker(ui, palette, state, ctrl, rocker_w, rocker_h, snap, now);

        // CENTER: MUTE & OPTIONS (Stacked Buttons)
        render_center_actions(ui, palette, state, ctrl, rocker_w, rocker_h, now);

        // RIGHT: CHANNEL VERTICAL ROCKER
        render_channel_rocker(ui, palette, state, ctrl, rocker_w, rocker_h, now);
    });
}

fn render_volume_rocker(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    w: f32,
    h: f32,
    snap: &Shared,
    now: f64,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let painter = ui.painter();
    let rocker_cr = (w * 0.45).clamp(24.0, 36.0);

    painter.rect_filled(rect, rocker_cr, palette.surface_high);
    painter.rect_stroke(rect, rocker_cr, palette.outline(), StrokeKind::Inside);

    // Dynamic fill bar for volume
    let vol_frac = match snap.volume {
        Some((lvl, max, _)) if max > 0 => (lvl as f32 / max as f32).clamp(0.0, 1.0),
        _ => state.volume_level.clamp(0.0, 1.0),
    };
    let fill_h = h * vol_frac;
    if fill_h > 0.0 {
        let fill_rect = Rect::from_min_max(pos2(rect.min.x, rect.max.y - fill_h), rect.max);
        painter.rect_filled(fill_rect, rocker_cr, Color32::from_rgba_unmultiplied(palette.primary.r(), palette.primary.g(), palette.primary.b(), 35));
    }

    let label_zone_h = (h * 0.26).clamp(28.0, 42.0);
    let btn_h = (h - label_zone_h) * 0.5;

    // Top: Vol +
    let top_rect = Rect::from_min_size(rect.min, vec2(w, btn_h));
    let top_resp = ui.interact(top_rect, ui.scope_id().with("vol_plus"), Sense::click());
    if top_resp.is_pointer_button_down_on() {
        painter.rect_filled(top_rect, rocker_cr * 0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
    }
    // Plus icon
    let plus_c = top_rect.center();
    let plus_size = (w * 0.12).clamp(6.5, 11.0);
    let stroke_thick = (w * 0.034).clamp(1.8, 2.6);
    painter.line_segment([pos2(plus_c.x - plus_size, plus_c.y), pos2(plus_c.x + plus_size, plus_c.y)], Stroke::new(stroke_thick, palette.text));
    painter.line_segment([pos2(plus_c.x, plus_c.y - plus_size), pos2(plus_c.x, plus_c.y + plus_size)], Stroke::new(stroke_thick, palette.text));

    let fire_plus = crate::ui::common::widgets::hold_logic(ui, &top_resp);
    if fire_plus {
        haptics::trigger(HapticStyle::Tick);
        ctrl.key(keys::VOLUME_UP);
        state.volume_level = (state.volume_level + 0.05).min(1.0);
        let pct = (state.volume_level * 100.0).round() as u32;
        state.send_command_feedback(format!("Volume {pct}%"), now);
    }

    // Center: VOL label + percentage
    let center_y = rect.center().y;
    let text_off = (label_zone_h * 0.20).clamp(6.0, 9.0);
    painter.text(
        pos2(rect.center().x, center_y - text_off),
        Align2::CENTER_CENTER,
        "VOL",
        FontId::proportional((w * 0.15).clamp(9.0, 11.5)),
        palette.text_muted,
    );
    let pct_text = format!("{}%", (vol_frac * 100.0).round() as u32);
    painter.text(
        pos2(rect.center().x, center_y + text_off),
        Align2::CENTER_CENTER,
        pct_text,
        FontId::proportional((w * 0.18).clamp(11.0, 14.0)),
        palette.primary,
    );

    // Bottom: Vol -
    let bot_rect = Rect::from_min_size(pos2(rect.min.x, rect.max.y - btn_h), vec2(w, btn_h));
    let bot_resp = ui.interact(bot_rect, ui.scope_id().with("vol_minus"), Sense::click());
    if bot_resp.is_pointer_button_down_on() {
        painter.rect_filled(bot_rect, rocker_cr * 0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
    }
    // Minus icon
    let minus_c = bot_rect.center();
    painter.line_segment([pos2(minus_c.x - plus_size, minus_c.y), pos2(minus_c.x + plus_size, minus_c.y)], Stroke::new(stroke_thick, palette.text));

    let fire_minus = crate::ui::common::widgets::hold_logic(ui, &bot_resp);
    if fire_minus {
        haptics::trigger(HapticStyle::Tick);
        ctrl.key(keys::VOLUME_DOWN);
        state.volume_level = (state.volume_level - 0.05).max(0.0);
        let pct = (state.volume_level * 100.0).round() as u32;
        state.send_command_feedback(format!("Volume {pct}%"), now);
    }
}

fn render_center_actions(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    w: f32,
    h: f32,
    now: f64,
) {
    let gap = (h * 0.06).clamp(4.0, 10.0);
    let btn_h = (h - gap) * 0.5;
    let btn_cr = (btn_h * 0.44).clamp(16.0, 26.0);
    let icon_dim = (btn_h * 0.35).clamp(14.0, 20.0);

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = gap;

        // Allocate both buttons first to avoid borrowing conflict
        let (mute_rect, mute_resp) = ui.allocate_exact_size(vec2(w, btn_h), Sense::click());
        let (opt_rect, opt_resp) = ui.allocate_exact_size(vec2(w, btn_h), Sense::click());

        // 1. Mute Toggle Button
        let (m_bg, m_fg, m_stroke) = if state.is_muted {
            (
                Color32::from_rgba_unmultiplied(244, 63, 94, 40),
                Color32::from_rgb(251, 113, 133),
                Stroke::new(1.4, Color32::from_rgb(244, 63, 94)),
            )
        } else {
            (palette.surface_low, palette.text, palette.outline())
        };

        let m_draw_rect = if mute_resp.is_pointer_button_down_on() { mute_rect.shrink(1.0) } else { mute_rect };
        ui.painter().rect_filled(m_draw_rect, btn_cr, m_bg);
        ui.painter().rect_stroke(m_draw_rect, btn_cr, m_stroke, StrokeKind::Inside);

        let mute_c = m_draw_rect.center();
        let off_y = (btn_h * 0.16).clamp(6.0, 9.0);
        let icon_box = Rect::from_center_size(mute_c + vec2(0.0, -off_y), vec2(icon_dim, icon_dim));
        svg_vol_mute(ui, icon_box);

        let mute_txt = if state.is_muted { "Muted" } else { "Mute" };
        ui.painter().text(
            mute_c + vec2(0.0, off_y),
            Align2::CENTER_CENTER,
            mute_txt,
            FontId::proportional((btn_h * 0.22).clamp(10.0, 12.5)),
            if state.is_muted { m_fg } else { palette.text_muted },
        );

        if mute_resp.clicked() {
            haptics::trigger(HapticStyle::Medium);
            state.is_muted = !state.is_muted;
            ctrl.key(keys::VOLUME_MUTE);
            let msg = if state.is_muted { "Audio Muted" } else { "Audio Unmuted" };
            state.send_command_feedback(msg, now);
        }

        // 2. Options / Quick Controls Trigger Button
        let opt_draw_rect = if opt_resp.is_pointer_button_down_on() { opt_rect.shrink(1.0) } else { opt_rect };
        let opt_bg = if opt_resp.hovered() { palette.surface_high } else { palette.surface_low };
        ui.painter().rect_filled(opt_draw_rect, btn_cr, opt_bg);
        ui.painter().rect_stroke(opt_draw_rect, btn_cr, palette.outline(), StrokeKind::Inside);

        let opt_c = opt_draw_rect.center();
        let tune_box = Rect::from_center_size(opt_c + vec2(0.0, -off_y), vec2(icon_dim, icon_dim));
        svg_tune(ui, tune_box, palette.text);

        ui.painter().text(
            opt_c + vec2(0.0, off_y),
            Align2::CENTER_CENTER,
            "Options",
            FontId::proportional((btn_h * 0.22).clamp(10.0, 12.5)),
            palette.text_muted,
        );

        if opt_resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            state.toggle_sheet(ActiveSheet::Options);
        }
    });
}

fn render_channel_rocker(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    w: f32,
    h: f32,
    now: f64,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let painter = ui.painter();
    let rocker_cr = (w * 0.45).clamp(24.0, 36.0);

    painter.rect_filled(rect, rocker_cr, palette.surface_high);
    painter.rect_stroke(rect, rocker_cr, palette.outline(), StrokeKind::Inside);

    let label_zone_h = (h * 0.26).clamp(28.0, 42.0);
    let btn_h = (h - label_zone_h) * 0.5;

    // Top: CH +
    let top_rect = Rect::from_min_size(rect.min, vec2(w, btn_h));
    let top_resp = ui.interact(top_rect, ui.scope_id().with("ch_plus"), Sense::click());
    if top_resp.is_pointer_button_down_on() {
        painter.rect_filled(top_rect, rocker_cr * 0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
    }
    let up_c = top_rect.center();
    let arr_w = (w * 0.10).clamp(5.5, 9.0);
    let arr_h = (w * 0.06).clamp(3.0, 5.5);
    let stroke_thick = (w * 0.034).clamp(1.8, 2.6);
    painter.add(Shape::line(
        vec![pos2(up_c.x - arr_w, up_c.y + arr_h), pos2(up_c.x, up_c.y - arr_h), pos2(up_c.x + arr_w, up_c.y + arr_h)],
        Stroke::new(stroke_thick, palette.text),
    ));

    let fire_ch_up = crate::ui::common::widgets::hold_logic(ui, &top_resp);
    if fire_ch_up {
        haptics::trigger(HapticStyle::Tick);
        ctrl.key(keys::CHANNEL_UP);
        state.current_channel += 1;
        state.numpad_buffer = state.current_channel.to_string();
        state.send_command_feedback(format!("Channel {}", state.current_channel), now);
    }

    // Center: CH label + channel number
    let center_y = rect.center().y;
    let text_off = (label_zone_h * 0.20).clamp(6.0, 9.0);
    painter.text(
        pos2(rect.center().x, center_y - text_off),
        Align2::CENTER_CENTER,
        "CH",
        FontId::proportional((w * 0.15).clamp(9.0, 11.5)),
        palette.text_muted,
    );
    let ch_text = state.current_channel.to_string();
    painter.text(
        pos2(rect.center().x, center_y + text_off),
        Align2::CENTER_CENTER,
        ch_text,
        FontId::proportional((w * 0.18).clamp(11.0, 14.5)),
        palette.primary,
    );

    // Bottom: CH -
    let bot_rect = Rect::from_min_size(pos2(rect.min.x, rect.max.y - btn_h), vec2(w, btn_h));
    let bot_resp = ui.interact(bot_rect, ui.scope_id().with("ch_minus"), Sense::click());
    if bot_resp.is_pointer_button_down_on() {
        painter.rect_filled(bot_rect, rocker_cr * 0.8, Color32::from_rgba_unmultiplied(255, 255, 255, 22));
    }
    let dn_c = bot_rect.center();
    painter.add(Shape::line(
        vec![pos2(dn_c.x - arr_w, dn_c.y - arr_h), pos2(dn_c.x, dn_c.y + arr_h), pos2(dn_c.x + arr_w, dn_c.y - arr_h)],
        Stroke::new(stroke_thick, palette.text),
    ));

    let fire_ch_down = crate::ui::common::widgets::hold_logic(ui, &bot_resp);
    if fire_ch_down {
        haptics::trigger(HapticStyle::Tick);
        ctrl.key(keys::CHANNEL_DOWN);
        if state.current_channel > 1 {
            state.current_channel -= 1;
        }
        state.numpad_buffer = state.current_channel.to_string();
        state.send_command_feedback(format!("Channel {}", state.current_channel), now);
    }
}
