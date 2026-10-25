//! Tab 3: Number Pad & Inputs for Material 3 Expressive UI.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::ui::state::UiState;
use crate::ui::theme::ThemePalette;
use eframe::egui::{pos2, vec2, Align2, FontId, Rect, Sense, StrokeKind, Ui};

pub fn render_expressive_keypad_tab(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    now: f64,
) {
    let avail_w = ui.available_width();

    // 1. Direct Channel Entry Preview Pill
    let preview_h = 44.0;
    let (prev_rect, _) = ui.allocate_exact_size(vec2(avail_w, preview_h), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(prev_rect, 16.0, palette.surface_low);
    painter.rect_stroke(prev_rect, 16.0, palette.outline(), StrokeKind::Inside);

    painter.text(
        pos2(prev_rect.min.x + 16.0, prev_rect.center().y),
        Align2::LEFT_CENTER,
        "Direct Channel Input",
        FontId::proportional(12.0),
        palette.text_muted,
    );

    let display_buf = if state.numpad_buffer.is_empty() {
        "_"
    } else {
        &state.numpad_buffer
    };
    painter.text(
        pos2(prev_rect.max.x - 16.0, prev_rect.center().y),
        Align2::RIGHT_CENTER,
        display_buf,
        FontId::proportional(18.0),
        palette.primary,
    );

    ui.add_space(10.0);

    // 2. 3x4 Rounded Number Pad
    let gap = 8.0;
    let btn_w = (avail_w - 2.0 * gap) / 3.0;
    let btn_h = (btn_w * 0.52).clamp(42.0, 58.0);

    let rows: [[&str; 3]; 4] = [
        ["1", "2", "3"],
        ["4", "5", "6"],
        ["7", "8", "9"],
        ["Guide", "0", "Go"],
    ];

    for row in rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for &item in &row {
                let (rect, resp) = ui.allocate_exact_size(vec2(btn_w, btn_h), Sense::click());
                let painter = ui.painter();
                let draw_rect = if resp.is_pointer_button_down_on() { rect.shrink(1.2) } else { rect };

                let (bg, fg) = match item {
                    "Go" => (palette.primary, palette.on_primary),
                    "Guide" => (palette.surface_low, palette.text_muted),
                    _ => (palette.surface_high, palette.text),
                };

                let btn_bg = if resp.is_pointer_button_down_on() {
                    match item {
                        "Go" => palette.primary_container,
                        _ => palette.surface_highest,
                    }
                } else if resp.hovered() {
                    match item {
                        "Go" => palette.primary,
                        _ => palette.surface_high,
                    }
                } else {
                    bg
                };

                painter.rect_filled(draw_rect, 24.0, btn_bg);
                if item != "Go" {
                    painter.rect_stroke(draw_rect, 24.0, palette.outline(), StrokeKind::Inside);
                }

                let c = draw_rect.center();
                match item {
                    "Guide" => {
                        let icon_box = Rect::from_center_size(c + vec2(0.0, -8.0), vec2(16.0, 16.0));
                        svg_guide(ui, icon_box, fg);
                        painter.text(c + vec2(0.0, 8.5), Align2::CENTER_CENTER, "Guide", FontId::proportional(10.5), fg);
                    }
                    "Go" => {
                        let icon_box = Rect::from_center_size(c + vec2(0.0, -7.0), vec2(16.0, 16.0));
                        draw_icon_return(painter, icon_box, fg);
                        painter.text(c + vec2(0.0, 8.5), Align2::CENTER_CENTER, "Go", FontId::proportional(11.0), fg);
                    }
                    digit => {
                        painter.text(c, Align2::CENTER_CENTER, digit, FontId::proportional(18.0), fg);
                    }
                }

                if resp.clicked() {
                    haptics::trigger(HapticStyle::Light);
                    match item {
                        "Guide" => {
                            ctrl.key(keys::GUIDE);
                            state.send_command_feedback("TV Guide", now);
                        }
                        "Go" => {
                            if !state.numpad_buffer.is_empty() {
                                if let Ok(ch) = state.numpad_buffer.parse::<u32>() {
                                    state.current_channel = ch;
                                    // Send digits to TV
                                    for ch_char in state.numpad_buffer.chars() {
                                        if let Some(k) = keys::key_for_char(ch_char) {
                                            ctrl.key(k);
                                        }
                                    }
                                    state.send_command_feedback(format!("Tuned to CH {ch}"), now);
                                }
                            }
                        }
                        digit => {
                            if state.numpad_buffer.len() >= 4 {
                                state.numpad_buffer.clear();
                            }
                            state.numpad_buffer.push_str(digit);
                            if let Ok(_ch) = state.numpad_buffer.parse::<u32>() {
                                if let Some(k) = keys::key_for_char(digit.chars().next().unwrap()) {
                                    ctrl.key(k);
                                }
                            }
                        }
                    }
                }
            }
        });
        ui.add_space(gap);
    }

    ui.add_space(4.0);

    // 3. Secondary Utility Strip (HDMI Input • Subtitles CC • Info)
    let util_h = 44.0;
    let util_w = (avail_w - 2.0 * gap) / 3.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;

        // Allocate all three buttons first
        let (in_rect, in_resp) = ui.allocate_exact_size(vec2(util_w, util_h), Sense::click());
        let (cc_rect, cc_resp) = ui.allocate_exact_size(vec2(util_w, util_h), Sense::click());
        let (info_rect, info_resp) = ui.allocate_exact_size(vec2(util_w, util_h), Sense::click());

        // 1. Input
        let in_bg = if in_resp.hovered() { palette.surface_high } else { palette.surface_low };
        ui.painter().rect_filled(in_rect, 16.0, in_bg);
        ui.painter().rect_stroke(in_rect, 16.0, palette.outline(), StrokeKind::Inside);
        let in_icon_box = Rect::from_center_size(pos2(in_rect.min.x + 18.0, in_rect.center().y), vec2(15.0, 15.0));
        svg_input(ui, in_icon_box, palette.text);
        ui.painter().text(pos2(in_rect.min.x + 32.0, in_rect.center().y), Align2::LEFT_CENTER, "Input", FontId::proportional(11.5), palette.text);
        if in_resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            ctrl.key(keys::TV_INPUT);
            state.send_command_feedback("HDMI Input", now);
        }

        // 2. Subtitles
        let cc_bg = if cc_resp.hovered() { palette.surface_high } else { palette.surface_low };
        ui.painter().rect_filled(cc_rect, 16.0, cc_bg);
        ui.painter().rect_stroke(cc_rect, 16.0, palette.outline(), StrokeKind::Inside);
        let cc_icon_box = Rect::from_center_size(pos2(cc_rect.min.x + 16.0, cc_rect.center().y), vec2(16.0, 14.0));
        draw_icon_closed_captions(ui.painter(), cc_icon_box, palette.text);
        ui.painter().text(pos2(cc_rect.min.x + 30.0, cc_rect.center().y), Align2::LEFT_CENTER, "Subtitles", FontId::proportional(11.0), palette.text);
        if cc_resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            ctrl.key(keys::CAPTIONS);
            state.send_command_feedback("Subtitles (CC)", now);
        }

        // 3. Info
        let info_bg = if info_resp.hovered() { palette.surface_high } else { palette.surface_low };
        ui.painter().rect_filled(info_rect, 16.0, info_bg);
        ui.painter().rect_stroke(info_rect, 16.0, palette.outline(), StrokeKind::Inside);
        let info_icon_box = Rect::from_center_size(pos2(info_rect.min.x + 18.0, info_rect.center().y), vec2(15.0, 15.0));
        svg_info(ui, info_icon_box, palette.text);
        ui.painter().text(pos2(info_rect.min.x + 32.0, info_rect.center().y), Align2::LEFT_CENTER, "Info", FontId::proportional(11.5), palette.text);
        if info_resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            ctrl.key(keys::INFO);
            state.send_command_feedback("Program Info", now);
        }
    });
}
