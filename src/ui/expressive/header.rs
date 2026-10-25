//! Unified Top Header for Material 3 Expressive UI.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::state::{ConnStatus, Shared};
use crate::ui::common::widgets::draw_live_status_dot;
use crate::ui::state::{ActiveSheet, UiState};
use crate::ui::theme::{ThemeKind, ThemePalette};
use crate::ui::UiMode;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

pub fn render_expressive_header(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    theme: &mut ThemeKind,
    ui_mode: &mut UiMode,
    now: f64,
) {
    let avail_w = ui.available_width();
    let header_h = ui.available_height().min(48.0).max(36.0);
    let btn_size = (header_h * 0.90).clamp(34.0, 42.0);
    let btn_gap = (avail_w * 0.016).clamp(4.0, 7.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = btn_gap;

        // 1. Consolidated Device Switcher Pill
        let is_connected = matches!(snap.status, ConnStatus::Connected);
        let active_name = ctrl
            .active
            .as_ref()
            .map(|t| t.name.clone())
            .or_else(|| ctrl.store.tvs.first().map(|t| t.name.clone()))
            .unwrap_or_else(|| "Living Room TV".to_string());

        let default_status = if !state.tv_powered {
            "Standby Mode"
        } else if is_connected {
            "Connected • Google TV"
        } else {
            "Offline • Tap to Connect"
        };
        let feedback_text = state.current_feedback(now, default_status);

        // 4 buttons on right: Power, Theme, Keyboard, Advanced
        let right_cluster_w = 4.0 * btn_size + 4.0 * btn_gap;
        let pill_w = (avail_w - right_cluster_w - btn_gap).clamp(130.0, 220.0);
        let (pill_rect, pill_resp) = ui.allocate_exact_size(vec2(pill_w, header_h), Sense::click());

        let pill_bg = if pill_resp.is_pointer_button_down_on() {
            palette.surface_highest
        } else if pill_resp.hovered() {
            palette.surface_high
        } else {
            palette.surface_low
        };

        let painter = ui.painter();
        painter.rect_filled(pill_rect, header_h * 0.5, pill_bg);
        painter.rect_stroke(pill_rect, header_h * 0.5, palette.outline(), StrokeKind::Inside);

        // Status dot
        let dot_c = pos2(pill_rect.min.x + 14.0, pill_rect.center().y);
        draw_live_status_dot(painter, dot_c, is_connected, !state.tv_powered, now);

        // TV Name & Subtitle
        let text_x = pill_rect.min.x + 26.0;

        let name_p = pos2(text_x, pill_rect.center().y - 8.0);
        painter.text(
            name_p,
            Align2::LEFT_CENTER,
            &active_name,
            FontId::proportional((header_h * 0.28).clamp(11.0, 13.5)),
            palette.text,
        );

        let fb_color = if now < state.feedback_until {
            palette.primary
        } else {
            palette.text_muted
        };
        let fb_p = pos2(text_x, pill_rect.center().y + 8.5);
        painter.text(
            fb_p,
            Align2::LEFT_CENTER,
            feedback_text,
            FontId::proportional((header_h * 0.22).clamp(9.0, 11.0)),
            fb_color,
        );

        // Expand chevron on right of pill
        let chevron_rect = Rect::from_center_size(pos2(pill_rect.max.x - 12.0, pill_rect.center().y), vec2(14.0, 14.0));
        draw_icon_expand_more(painter, chevron_rect, palette.text_muted);

        if pill_resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            state.toggle_sheet(ActiveSheet::Device);
        }

        // Right Utility Actions: Power, Theme, Keyboard, Advanced Mode
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = btn_gap;

            let icon_btn_size = vec2(btn_size, btn_size);
            let (power_rect, power_resp) = ui.allocate_exact_size(icon_btn_size, Sense::click());
            let (theme_rect, theme_resp) = ui.allocate_exact_size(icon_btn_size, Sense::click());
            let (kb_rect, kb_resp) = ui.allocate_exact_size(icon_btn_size, Sense::click());
            let (adv_rect, adv_resp) = ui.allocate_exact_size(icon_btn_size, Sense::click());

            // 1. Power Button
            let (p_bg, p_fg, p_border) = if state.tv_powered {
                (
                    Color32::from_rgba_unmultiplied(244, 63, 94, 38),
                    Color32::from_rgb(251, 113, 133), // rose
                    Color32::from_rgba_unmultiplied(244, 63, 94, 70),
                )
            } else {
                (
                    Color32::from_rgba_unmultiplied(16, 185, 129, 45),
                    Color32::from_rgb(52, 211, 153), // emerald
                    Color32::from_rgba_unmultiplied(16, 185, 129, 80),
                )
            };

            let p_draw_rect = if power_resp.is_pointer_button_down_on() {
                power_rect.shrink(1.2)
            } else {
                power_rect
            };

            ui.painter().rect_filled(p_draw_rect, btn_size * 0.5, p_bg);
            ui.painter().rect_stroke(p_draw_rect, btn_size * 0.5, Stroke::new(1.2, p_border), StrokeKind::Inside);
            let icon_dim = (btn_size * 0.46).clamp(15.0, 20.0);
            draw_icon_power_ring(ui.painter(), Rect::from_center_size(p_draw_rect.center(), vec2(icon_dim * 1.05, icon_dim * 1.05)), p_fg);

            if power_resp.clicked() {
                haptics::trigger(HapticStyle::Heavy);
                state.tv_powered = !state.tv_powered;
                ctrl.key(keys::POWER);
                let msg = if state.tv_powered { "TV Powered On" } else { "TV Standby" };
                state.send_command_feedback(msg, now);
            }

            // 2. Theme Switcher Button
            let t_bg = if theme_resp.is_pointer_button_down_on() {
                palette.surface_highest
            } else if theme_resp.hovered() {
                palette.surface_high
            } else {
                palette.surface_low
            };
            let t_draw_rect = if theme_resp.is_pointer_button_down_on() {
                theme_rect.shrink(1.0)
            } else {
                theme_rect
            };
            ui.painter().rect_filled(t_draw_rect, btn_size * 0.5, t_bg);
            ui.painter().rect_stroke(t_draw_rect, btn_size * 0.5, palette.outline(), StrokeKind::Inside);
            svg_palette(ui, Rect::from_center_size(t_draw_rect.center(), vec2(icon_dim, icon_dim)), palette.primary);

            if theme_resp.clicked() {
                haptics::trigger(HapticStyle::Light);
                *theme = theme.next();
                state.send_command_feedback(format!("Theme: {}", theme.name()), now);
            }

            // 3. Quick TV Keyboard Button
            let kb_bg = if kb_resp.is_pointer_button_down_on() {
                palette.surface_highest
            } else if kb_resp.hovered() {
                palette.surface_high
            } else {
                palette.surface_low
            };
            let kb_draw_rect = if kb_resp.is_pointer_button_down_on() {
                kb_rect.shrink(1.0)
            } else {
                kb_rect
            };
            ui.painter().rect_filled(kb_draw_rect, btn_size * 0.5, kb_bg);
            ui.painter().rect_stroke(kb_draw_rect, btn_size * 0.5, palette.outline(), StrokeKind::Inside);
            svg_keyboard(ui, Rect::from_center_size(kb_draw_rect.center(), vec2(icon_dim, icon_dim)), palette.text);

            if kb_resp.clicked() {
                haptics::trigger(HapticStyle::Light);
                state.toggle_sheet(ActiveSheet::Keyboard);
            }

            // 4. Advanced Mode Toggle Pill/Button
            let adv_bg = if adv_resp.is_pointer_button_down_on() {
                palette.primary_container
            } else if adv_resp.hovered() {
                palette.surface_high
            } else {
                palette.surface_low
            };
            let adv_draw_rect = if adv_resp.is_pointer_button_down_on() {
                adv_rect.shrink(1.0)
            } else {
                adv_rect
            };
            ui.painter().rect_filled(adv_draw_rect, btn_size * 0.5, adv_bg);
            ui.painter().rect_stroke(adv_draw_rect, btn_size * 0.5, palette.outline(), StrokeKind::Inside);
            svg_tune(ui, Rect::from_center_size(adv_draw_rect.center(), vec2(icon_dim, icon_dim)), palette.text_muted);

            if adv_resp.clicked() {
                haptics::trigger(HapticStyle::Heavy);
                *ui_mode = ui_mode.next();
                state.send_command_feedback(format!("Layout: {}", ui_mode.name()), now);
            }
        });
    });
}
