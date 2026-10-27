//! Minimal Apple TV Edition (Primary Default UI) for FryRemote.
//! Sculpted hardware chassis, iconic circular clickpad with touch & swipe,
//! 2-column tactile controls with continuous volume rocker, and minimal overlay sheets.

pub mod clickpad;
pub mod sheets;

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::state::{ConnStatus, Shared};
use crate::ui::common::widgets::hold_logic;
use crate::ui::state::{ActiveSheet, UiState};
use crate::ui::theme::{ThemeKind, ThemePalette};
use crate::ui::UiMode;
use clickpad::{draw_apple_clickpad, ClickpadAction};
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

pub fn render_minimal_ui(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    theme: &mut ThemeKind,
    ui_mode: &mut UiMode,
    screen: &mut crate::app::Screen,
) {
    let now = ui.input(|i| i.time);
    let total_rect = ui.max_rect();
    let total_w = total_rect.width();
    let total_h = total_rect.height();
    let is_desktop_wide = total_w > 480.0;

    // Canvas background
    ui.painter().rect_filled(total_rect, 0.0, palette.bg);

    // Sculpted Remote Chassis Sizing (Fully Adaptive across all form factors)
    let chassis_w = if is_desktop_wide {
        (total_w * 0.40).clamp(360.0, 460.0)
    } else {
        let margin_x = if total_w < 380.0 { 10.0 } else { 14.0 };
        (total_w - 2.0 * margin_x).clamp(280.0, 430.0)
    };
    let pad_x = ((total_w - chassis_w) * 0.5).max(0.0);

    // Desktop Top Presentation Bar (Wide Desktop Window)
    if is_desktop_wide {
        let bar_h = 32.0;
        let bar_rect = Rect::from_min_size(pos2(pad_x, total_rect.min.y + 6.0), vec2(chassis_w, bar_h));
        let mut bar_ui = ui.new_child(egui::UiBuilder::new().max_rect(bar_rect));

        bar_ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let (dot_rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 4.0, palette.primary);
            ui.label(
                eframe::egui::RichText::new("Apple TV Minimal")
                    .strong()
                    .size(12.0)
                    .color(palette.text),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                // Layout cycle button
                let layout_btn = eframe::egui::RichText::new(format!("Mode: {}", ui_mode.name()))
                    .size(11.0)
                    .color(palette.primary);
                if ui.button(layout_btn).clicked() {
                    haptics::trigger(HapticStyle::Heavy);
                    *ui_mode = ui_mode.next();
                    state.send_command_feedback(format!("Layout: {}", ui_mode.name()), now);
                }

                // Finish dots (Obsidian, Silver, Titanium)
                let finishes = [ThemeKind::Obsidian, ThemeKind::Silver, ThemeKind::Titanium];
                for f in finishes {
                    let is_sel = *theme == f;
                    let (dot_rect, resp) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
                    ui.painter().circle_filled(dot_rect.center(), 7.0, f.dot_color());
                    if is_sel {
                        ui.painter().circle_stroke(
                            dot_rect.center(),
                            8.5,
                            Stroke::new(1.8, Color32::WHITE),
                        );
                    }
                    if resp.clicked() {
                        haptics::trigger(HapticStyle::Light);
                        *theme = f;
                        state.send_command_feedback(format!("Finish: {}", f.name()), now);
                    }
                }
            });
        });
    }

    // Remote Chassis Rect
    let chassis_top = if is_desktop_wide {
        total_rect.min.y + 44.0
    } else {
        let margin_y = if total_h < 700.0 { 6.0 } else { 10.0 };
        total_rect.min.y + margin_y
    };
    let chassis_h = if is_desktop_wide {
        (total_h - 56.0).min(920.0)
    } else {
        let margin_y = if total_h < 700.0 { 6.0 } else { 10.0 };
        total_h - 2.0 * margin_y
    };

    let chassis_rect = Rect::from_min_size(pos2(pad_x, chassis_top), vec2(chassis_w, chassis_h));
    let chassis_cr = (chassis_w * 0.135).clamp(42.0, 56.0);

    ui.painter().rect_filled(chassis_rect, chassis_cr, palette.surface);
    ui.painter().rect_stroke(
        chassis_rect,
        chassis_cr,
        Stroke::new(1.2, palette.outline_color),
        StrokeKind::Inside,
    );

    // Subtle inner chassis rim highlight
    let inner_rim = chassis_rect.shrink(1.0);
    ui.painter().rect_stroke(
        inner_rim,
        chassis_cr - 1.0,
        Stroke::new(0.6, Color32::from_rgba_unmultiplied(255, 255, 255, 12)),
        StrokeKind::Inside,
    );

    // Side On-Screen Volume Toast Pill
    if now < state.volume_toast_until {
        let toast_w = 4.0;
        let toast_h = (chassis_h * 0.14).clamp(80.0, 120.0);
        let toast_rect = Rect::from_center_size(
            pos2(chassis_rect.max.x - 10.0, chassis_rect.center().y),
            vec2(toast_w, toast_h),
        );
        ui.painter().rect_filled(toast_rect, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30));
        let fill_h = toast_h * state.volume_level.clamp(0.0, 1.0);
        if fill_h > 0.0 {
            let fill_rect = Rect::from_min_max(
                pos2(toast_rect.min.x, toast_rect.max.y - fill_h),
                toast_rect.max,
            );
            ui.painter().rect_filled(fill_rect, 2.0, palette.primary);
        }
    }

    // Chassis Content Bounds
    let content_insets = vec2(
        (chassis_w * 0.052).clamp(12.0, 20.0),
        (chassis_h * 0.018).clamp(8.0, 16.0),
    );
    let inner_rect = chassis_rect.shrink2(content_insets);
    let inner_w = inner_rect.width();

    // =========================================================================
    // 1. TOP ROW: SUBTLE TV PILL & POWER BUTTON
    // =========================================================================
    let header_h = (chassis_h * 0.052).clamp(34.0, 44.0);
    let header_rect = Rect::from_min_size(inner_rect.min, vec2(inner_w, header_h));

    // Left: Minimal Status Pill
    let is_connected = matches!(snap.status, ConnStatus::Connected);
    let active_name = ctrl
        .active
        .as_ref()
        .map(|t| t.name.clone())
        .or_else(|| ctrl.store.tvs.first().map(|t| t.name.clone()));

    let default_status = if !state.tv_powered {
        "Standby".to_string()
    } else if is_connected {
        active_name.unwrap_or_else(|| "Connected".to_string())
    } else if let Some(name) = active_name {
        format!("{name} • Offline")
    } else {
        "No TV • Tap to Scan".to_string()
    };
    let pill_text = state.current_feedback(now, &default_status);

    let pill_w = (inner_w * 0.48).clamp(130.0, 185.0);
    let pill_rect = Rect::from_min_size(header_rect.min, vec2(pill_w, header_h));
    let pill_resp = ui.interact(pill_rect, ui.scope_id().with("minimal_device_pill"), Sense::click());

    let pill_bg = if pill_resp.hovered() {
        Color32::from_rgba_unmultiplied(128, 128, 128, 40)
    } else {
        Color32::from_rgba_unmultiplied(128, 128, 128, 25)
    };
    ui.painter().rect_filled(pill_rect, header_h * 0.5, pill_bg);

    // Status LED
    let led_c = pos2(pill_rect.min.x + 14.0, pill_rect.center().y);
    let led_color = if !state.tv_powered {
        Color32::from_rgb(251, 191, 36) // amber
    } else if is_connected {
        Color32::from_rgb(52, 211, 153) // emerald
    } else {
        Color32::from_rgb(248, 113, 113) // red
    };
    ui.painter().circle_filled(led_c, 3.2, led_color);

    let text_color = if now < state.feedback_until {
        palette.primary
    } else {
        palette.text_muted
    };
    let pill_font_size = (header_h * 0.29).clamp(10.5, 13.0);
    ui.painter().text(
        pos2(pill_rect.min.x + 25.0, pill_rect.center().y),
        Align2::LEFT_CENTER,
        pill_text,
        FontId::proportional(pill_font_size),
        text_color,
    );

    if pill_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.toggle_sheet(ActiveSheet::Device);
    }

    // Right: Circular Power Button
    let p_btn_size = header_h;
    let power_rect = Rect::from_center_size(
        pos2(header_rect.max.x - p_btn_size * 0.5, header_rect.center().y),
        vec2(p_btn_size, p_btn_size),
    );
    let power_resp = ui.interact(power_rect, ui.scope_id().with("minimal_power"), Sense::click());

    let p_draw_rect = if power_resp.is_pointer_button_down_on() {
        power_rect.shrink(1.2)
    } else {
        power_rect
    };
    let p_bg = if power_resp.hovered() { palette.surface_highest } else { palette.surface_high };
    ui.painter().circle_filled(p_draw_rect.center(), p_btn_size * 0.5, p_bg);
    ui.painter().circle_stroke(p_draw_rect.center(), p_btn_size * 0.5, Stroke::new(1.0, palette.outline_color));

    let power_icon_dim = (p_btn_size * 0.44).clamp(14.0, 20.0);
    let power_icon_rect = Rect::from_center_size(p_draw_rect.center(), vec2(power_icon_dim, power_icon_dim));
    let p_fg = if state.tv_powered {
        Color32::from_rgb(251, 113, 133) // rose
    } else {
        Color32::from_rgb(52, 211, 153) // emerald
    };
    draw_icon_power_ring(ui.painter(), power_icon_rect, p_fg);

    if power_resp.clicked() {
        haptics::trigger(HapticStyle::Heavy);
        state.tv_powered = !state.tv_powered;
        ctrl.key(keys::POWER);
        let msg = if state.tv_powered { "Power On" } else { "Standby" };
        state.send_command_feedback(msg, now);
    }

    // =========================================================================
    // 4. MINIMAL UTILITY FOOTER (PINNED AT BOTTOM)
    // =========================================================================
    let footer_h = (chassis_h * 0.052).clamp(34.0, 44.0);
    let footer_rect = Rect::from_min_size(
        pos2(inner_rect.min.x, inner_rect.max.y - footer_h),
        vec2(inner_w, footer_h),
    );
    let foot_btn_size = (footer_h * 0.90).clamp(30.0, 40.0);
    let foot_icon_size = (foot_btn_size * 0.50).clamp(15.0, 20.0);

    // Left: Quick Apps Drawer (circular button)
    let apps_rect = Rect::from_center_size(
        pos2(footer_rect.min.x + foot_btn_size * 0.5, footer_rect.center().y),
        vec2(foot_btn_size, foot_btn_size),
    );
    let apps_resp = ui.interact(apps_rect, ui.scope_id().with("minimal_apps_btn"), Sense::click());
    if apps_resp.hovered() {
        ui.painter().circle_filled(apps_rect.center(), foot_btn_size * 0.5, palette.surface_high);
    }
    let apps_fg = if apps_resp.hovered() { palette.text } else { palette.text_muted };
    svg_nav_apps(ui, Rect::from_center_size(apps_rect.center(), vec2(foot_icon_size, foot_icon_size)), apps_fg);
    if apps_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.toggle_sheet(ActiveSheet::Apps);
    }

    // Center: Dedicated Siri / Google Assistant Mic Pill
    let mic_w = (inner_w * 0.28).clamp(80.0, 110.0);
    let mic_h = (footer_h * 0.76).clamp(26.0, 34.0);
    let mic_rect = Rect::from_center_size(footer_rect.center(), vec2(mic_w, mic_h));
    let mic_resp = ui.interact(mic_rect, ui.scope_id().with("minimal_voice_pill"), Sense::click());
    let mic_bg = if mic_resp.is_pointer_button_down_on() {
        palette.surface_highest
    } else {
        Color32::from_rgba_unmultiplied(128, 128, 128, 25)
    };
    ui.painter().rect_filled(mic_rect, mic_h * 0.5, mic_bg);

    let mic_icon_dim = (mic_h * 0.46).clamp(12.0, 16.0);
    let mic_icon_rect = Rect::from_center_size(pos2(mic_rect.min.x + mic_h * 0.55, mic_rect.center().y), vec2(mic_icon_dim, mic_icon_dim));
    svg_mic(ui, mic_icon_rect, palette.primary);
    ui.painter().text(
        pos2(mic_rect.min.x + mic_h * 1.05, mic_rect.center().y),
        Align2::LEFT_CENTER,
        "Voice",
        FontId::proportional((mic_h * 0.38).clamp(10.0, 13.0)),
        palette.text_muted,
    );
    if mic_resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        state.toggle_sheet(ActiveSheet::Voice);
    }

    // Right cluster: Finish Switcher + Layout Loop Switcher
    let finish_rect = Rect::from_center_size(
        pos2(footer_rect.max.x - foot_btn_size * 1.35, footer_rect.center().y),
        vec2(foot_btn_size, foot_btn_size),
    );
    let finish_resp = ui.interact(finish_rect, ui.scope_id().with("minimal_finish_btn"), Sense::click());
    if finish_resp.hovered() {
        ui.painter().circle_filled(finish_rect.center(), foot_btn_size * 0.5, palette.surface_high);
    }
    let f_fg = if finish_resp.hovered() { palette.text } else { palette.text_muted };
    svg_palette(ui, Rect::from_center_size(finish_rect.center(), vec2(foot_icon_size, foot_icon_size)), f_fg);
    if finish_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        *theme = match *theme {
            ThemeKind::Obsidian => ThemeKind::Silver,
            ThemeKind::Silver => ThemeKind::Titanium,
            _ => ThemeKind::Obsidian,
        };
        state.send_command_feedback(format!("Finish: {}", theme.name()), now);
    }

    // Layout Loop Switcher (cycles Minimal -> Expressive -> Advanced -> Minimal)
    let layout_rect = Rect::from_center_size(
        pos2(footer_rect.max.x - foot_btn_size * 0.45, footer_rect.center().y),
        vec2(foot_btn_size, foot_btn_size),
    );
    let layout_resp = ui.interact(layout_rect, ui.scope_id().with("minimal_layout_btn"), Sense::click());
    if layout_resp.hovered() {
        ui.painter().circle_filled(layout_rect.center(), 16.0, palette.surface_high);
    }
    let l_fg = if layout_resp.hovered() { palette.primary } else { palette.text_muted };
    svg_tune(ui, Rect::from_center_size(layout_rect.center(), vec2(17.0, 17.0)), l_fg);
    if layout_resp.clicked() {
        haptics::trigger(HapticStyle::Heavy);
        *ui_mode = ui_mode.next();
        state.send_command_feedback(format!("Layout: {}", ui_mode.name()), now);
    }

    // =========================================================================
    // 2. ICONIC CIRCULAR CLICKPAD & 3. 2-COLUMN ESSENTIAL CONTROLS
    // =========================================================================
    let middle_top = header_rect.max.y;
    let middle_bot = footer_rect.min.y;
    let middle_rect = Rect::from_min_max(
        pos2(inner_rect.min.x, middle_top),
        pos2(inner_rect.max.x, middle_bot),
    );
    let middle_w = inner_w;
    let middle_h = (middle_bot - middle_top).max(200.0);

    let mut middle_ui = ui.new_child(egui::UiBuilder::new().max_rect(middle_rect));

    // Dynamic scale factor based on both available height and width
    let h_ratio = (middle_h / 560.0).clamp(0.72, 1.35);
    let w_ratio = (middle_w / 310.0).clamp(0.78, 1.35);
    let dyn_scale = h_ratio * 0.65 + w_ratio * 0.35;

    // Proportional element sizing
    let pad_size = (220.0 * dyn_scale)
        .clamp(168.0, 275.0)
        .min(middle_w * 0.86)
        .min(middle_h * 0.44);

    let gap_x = (middle_w * 0.08).clamp(18.0, 34.0);
    let col_w = (66.0 * dyn_scale)
        .clamp(52.0, 82.0)
        .min((middle_w - gap_x - 24.0) * 0.5);
    let gap_rows = (14.0 * dyn_scale).clamp(8.0, 20.0);

    let controls_h = col_w * 3.0 + gap_rows * 2.0;
    let total_content_h = pad_size + controls_h;

    // Remaining vertical space is distributed gracefully
    let extra_h = (middle_h - total_content_h).max(0.0);
    let top_pad_space = (extra_h * 0.16).clamp(4.0, 32.0);
    let gap_pad_controls = (extra_h * 0.44).clamp(10.0, 56.0);

    // A. Clickpad Assembly
    let pad_top = middle_rect.min.y + top_pad_space;
    let _pad_center_y = pad_top + pad_size * 0.5;
    let pad_pos = pos2(middle_rect.center().x - pad_size * 0.5, pad_top);

    let mut clickpad_ui = middle_ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(pad_pos, vec2(pad_size, pad_size))));
    let click_act = draw_apple_clickpad(&mut clickpad_ui, palette, state, pad_size, now);

    if let Some(act) = click_act {
        match act {
            ClickpadAction::Up => {
                ctrl.key(keys::DPAD_UP);
                state.send_command_feedback("Up", now);
            }
            ClickpadAction::Down => {
                ctrl.key(keys::DPAD_DOWN);
                state.send_command_feedback("Down", now);
            }
            ClickpadAction::Left => {
                ctrl.key(keys::DPAD_LEFT);
                state.send_command_feedback("Left", now);
            }
            ClickpadAction::Right => {
                ctrl.key(keys::DPAD_RIGHT);
                state.send_command_feedback("Right", now);
            }
            ClickpadAction::Center => {
                ctrl.key(keys::DPAD_CENTER);
                state.send_command_feedback("Select", now);
            }
            ClickpadAction::SwipeUp => {
                ctrl.key(keys::DPAD_UP);
                state.send_command_feedback("Swipe Up", now);
            }
            ClickpadAction::SwipeDown => {
                ctrl.key(keys::DPAD_DOWN);
                state.send_command_feedback("Swipe Down", now);
            }
            ClickpadAction::SwipeLeft => {
                ctrl.key(keys::DPAD_LEFT);
                state.send_command_feedback("Swipe Left", now);
            }
            ClickpadAction::SwipeRight => {
                ctrl.key(keys::DPAD_RIGHT);
                state.send_command_feedback("Swipe Right", now);
            }
        }
    }

    // B. Essential 2-Column Controls
    let controls_top = pad_top + pad_size + gap_pad_controls;
    let grid_w = col_w * 2.0 + gap_x;
    let grid_start_x = middle_rect.center().x - grid_w * 0.5;

    let left_col_center_x = grid_start_x + col_w * 0.5;
    let right_col_center_x = grid_start_x + col_w + gap_x + col_w * 0.5;

    // Row 1: Back (<) & TV / Home
    let r1_y = controls_top + col_w * 0.5;
    let icon_dim = (col_w * 0.35).clamp(16.0, 28.0);

    // 1. Back button (<)
    let back_rect = Rect::from_center_size(pos2(left_col_center_x, r1_y), vec2(col_w, col_w));
    let back_resp = middle_ui.interact(back_rect, middle_ui.scope_id().with("apple_back"), Sense::click());
    let b_draw_rect = if back_resp.is_pointer_button_down_on() { back_rect.shrink(1.2) } else { back_rect };
    let b_bg = if back_resp.hovered() { palette.surface_highest } else { palette.surface_high };
    middle_ui.painter().circle_filled(b_draw_rect.center(), col_w * 0.5, b_bg);
    middle_ui.painter().circle_stroke(b_draw_rect.center(), col_w * 0.5, Stroke::new(1.0, palette.outline_color));
    svg_back(&middle_ui, Rect::from_center_size(b_draw_rect.center(), vec2(icon_dim, icon_dim)), palette.text);
    if back_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        ctrl.key(keys::BACK);
        state.send_command_feedback("Back", now);
    }

    // 2. TV / Home button
    let home_rect = Rect::from_center_size(pos2(right_col_center_x, r1_y), vec2(col_w, col_w));
    let home_resp = middle_ui.interact(home_rect, middle_ui.scope_id().with("apple_home"), Sense::click());
    let h_draw_rect = if home_resp.is_pointer_button_down_on() { home_rect.shrink(1.2) } else { home_rect };
    let h_bg = if home_resp.hovered() { palette.surface_highest } else { palette.surface_high };
    middle_ui.painter().circle_filled(h_draw_rect.center(), col_w * 0.5, h_bg);
    middle_ui.painter().circle_stroke(h_draw_rect.center(), col_w * 0.5, Stroke::new(1.0, palette.outline_color));
    svg_tv(&middle_ui, Rect::from_center_size(h_draw_rect.center(), vec2(icon_dim * 1.08, icon_dim * 1.08)), palette.text);
    if home_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        ctrl.key(keys::HOME);
        state.send_command_feedback("Home", now);
    }

    // Rows 2 & 3: Left Column (Play/Pause + Mute) | Right Column (Single Continuous Volume Rocker)
    let r2_top = controls_top + col_w + gap_rows;
    let rocker_h = col_w * 2.0 + gap_rows; // matches left stack height exactly!

    // Left Column Top: Play / Pause
    let pp_rect = Rect::from_center_size(pos2(left_col_center_x, r2_top + col_w * 0.5), vec2(col_w, col_w));
    let pp_resp = middle_ui.interact(pp_rect, middle_ui.scope_id().with("apple_play_pause"), Sense::click());
    let pp_draw = if pp_resp.is_pointer_button_down_on() { pp_rect.shrink(1.2) } else { pp_rect };
    let pp_bg = if pp_resp.hovered() { palette.surface_highest } else { palette.surface_high };
    middle_ui.painter().circle_filled(pp_draw.center(), col_w * 0.5, pp_bg);
    middle_ui.painter().circle_stroke(pp_draw.center(), col_w * 0.5, Stroke::new(1.0, palette.outline_color));
    svg_play_pause(&middle_ui, Rect::from_center_size(pp_draw.center(), vec2(icon_dim * 1.05, icon_dim * 1.05)), palette.text);
    if pp_resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        state.is_playing = !state.is_playing;
        ctrl.key(keys::MEDIA_PLAY_PAUSE);
        let msg = if state.is_playing { "Play" } else { "Pause" };
        state.send_command_feedback(msg, now);
    }

    // Left Column Bottom: Mute Toggle
    let mute_y = r2_top + col_w + gap_rows + col_w * 0.5;
    let mute_rect = Rect::from_center_size(pos2(left_col_center_x, mute_y), vec2(col_w, col_w));
    let mute_resp = middle_ui.interact(mute_rect, middle_ui.scope_id().with("apple_mute"), Sense::click());
    let m_draw = if mute_resp.is_pointer_button_down_on() { mute_rect.shrink(1.2) } else { mute_rect };
    let (m_bg, _m_fg, m_stroke) = if state.is_muted {
        (
            Color32::from_rgba_unmultiplied(244, 63, 94, 35),
            Color32::from_rgb(251, 113, 133), // rose
            Stroke::new(1.4, Color32::from_rgb(244, 63, 94)),
        )
    } else {
        (
            if mute_resp.hovered() { palette.surface_highest } else { palette.surface_high },
            palette.text,
            Stroke::new(1.0, palette.outline_color),
        )
    };
    middle_ui.painter().circle_filled(m_draw.center(), col_w * 0.5, m_bg);
    middle_ui.painter().circle_stroke(m_draw.center(), col_w * 0.5, m_stroke);
    svg_vol_mute(&middle_ui, Rect::from_center_size(m_draw.center(), vec2(icon_dim, icon_dim)));
    if mute_resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        state.is_muted = !state.is_muted;
        ctrl.key(keys::VOLUME_MUTE);
        let msg = if state.is_muted { "Muted" } else { "Unmuted" };
        state.send_command_feedback(msg, now);
    }

    // Right Column: Sculpted Vertical Volume Rocker (+ / -)
    let rocker_rect = Rect::from_min_size(pos2(grid_start_x + col_w + gap_x, r2_top), vec2(col_w, rocker_h));
    middle_ui.painter().rect_filled(rocker_rect, col_w * 0.5, palette.surface_high);
    middle_ui.painter().rect_stroke(rocker_rect, col_w * 0.5, Stroke::new(1.0, palette.outline_color), StrokeKind::Inside);

    let half_h = rocker_h * 0.5;

    // Top half: Volume Up (+)
    let vol_up_rect = Rect::from_min_size(rocker_rect.min, vec2(col_w, half_h));
    let vol_up_resp = middle_ui.interact(vol_up_rect, middle_ui.scope_id().with("apple_vol_up"), Sense::click());
    if vol_up_resp.is_pointer_button_down_on() {
        middle_ui.painter().rect_filled(vol_up_rect, col_w * 0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 20));
    }
    // Plus glyph
    let plus_c = vol_up_rect.center();
    let plus_size = (col_w * 0.13).clamp(7.0, 11.0);
    let glyph_stroke = (col_w * 0.034).clamp(1.8, 2.6);
    middle_ui.painter().line_segment([pos2(plus_c.x - plus_size, plus_c.y), pos2(plus_c.x + plus_size, plus_c.y)], Stroke::new(glyph_stroke, palette.text));
    middle_ui.painter().line_segment([pos2(plus_c.x, plus_c.y - plus_size), pos2(plus_c.x, plus_c.y + plus_size)], Stroke::new(glyph_stroke, palette.text));

    if hold_logic(&middle_ui, &vol_up_resp) {
        haptics::trigger(HapticStyle::Tick);
        if state.is_muted {
            state.is_muted = false;
        }
        ctrl.key(keys::VOLUME_UP);
        state.volume_level = (state.volume_level + 0.05).min(1.0);
        state.volume_toast_until = now + 1.2;
        let pct = (state.volume_level * 100.0).round() as u32;
        state.send_command_feedback(format!("Volume {pct}%"), now);
    }

    // Center divider line
    let div_c = rocker_rect.center();
    let div_half = (col_w * 0.22).clamp(10.0, 18.0);
    middle_ui.painter().line_segment(
        [pos2(div_c.x - div_half, div_c.y), pos2(div_c.x + div_half, div_c.y)],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 25)),
    );

    // Bottom half: Volume Down (-)
    let vol_down_rect = Rect::from_min_size(pos2(rocker_rect.min.x, div_c.y), vec2(col_w, half_h));
    let vol_down_resp = middle_ui.interact(vol_down_rect, middle_ui.scope_id().with("apple_vol_down"), Sense::click());
    if vol_down_resp.is_pointer_button_down_on() {
        middle_ui.painter().rect_filled(vol_down_rect, col_w * 0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 20));
    }
    // Minus glyph
    let minus_c = vol_down_rect.center();
    middle_ui.painter().line_segment([pos2(minus_c.x - plus_size, minus_c.y), pos2(minus_c.x + plus_size, minus_c.y)], Stroke::new(glyph_stroke, palette.text));

    if hold_logic(&middle_ui, &vol_down_resp) {
        haptics::trigger(HapticStyle::Tick);
        ctrl.key(keys::VOLUME_DOWN);
        state.volume_level = (state.volume_level - 0.05).max(0.0);
        state.volume_toast_until = now + 1.2;
        let pct = (state.volume_level * 100.0).round() as u32;
        state.send_command_feedback(format!("Volume {pct}%"), now);
    }

    // =========================================================================
    // 5. MINIMAL OVERLAYS & SHEETS
    // =========================================================================
    sheets::render_minimal_sheets(ui, palette, state, ctrl, snap, screen, now);
}
