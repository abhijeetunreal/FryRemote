//! Advanced Mode Remote Tab: Full-screen responsive layout utilizing all available space.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::state::Shared;
use crate::ui::common::dpad::draw_responsive_clover_dpad;
use crate::ui::common::widgets::{custom_button, custom_button_stacked, BtnIcon};
use crate::ui::state::UiState;
use eframe::egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

pub const CARD_BG: Color32 = Color32::from_rgb(15, 23, 42);
pub const BORDER: Color32 = Color32::from_rgb(28, 43, 68);
pub const ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
pub const TEXT_WHITE: Color32 = Color32::from_rgb(248, 250, 252);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
pub const ICON_SLATE: Color32 = Color32::from_rgb(148, 163, 184);

pub fn render_advanced_remote_tab(
    ui: &mut Ui,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    w: f32,
    _now: f64,
) {
    // 1. Power Button & Volume Slider Card Row (Full width)
    render_power_volume_row(ui, state, ctrl, snap, w);
    ui.add_space(8.0);

    // 2. Responsive Clover D-Pad
    render_scalable_dpad(ui, state, ctrl, w);
    ui.add_space(8.0);

    // 3. Navigation Row (Back, Home, Menu)
    render_nav_row(ui, ctrl, w);
    ui.add_space(6.0);

    // 4. Function Grid Row 1 (Vol -, Mute, Vol +, Input)
    render_function_row_1(ui, ctrl, w);
    ui.add_space(6.0);

    // 5. Function Grid Row 2 (Ch -, Ch +, Guide, Info)
    render_function_row_2(ui, ctrl, w);
    ui.add_space(6.0);

    // 6. Media Grid Row 3 (Prev, Play/Pause, Next)
    render_function_row_3(ui, ctrl, w);
    ui.add_space(6.0);

    // 7. Utility Grid Row 4 (Search, Captions, Settings)
    render_function_row_4(ui, ctrl, w);
    ui.add_space(8.0);

    // 8. Apps Section (3 Columns x 2 Rows)
    render_apps_section(ui, ctrl, w);
    ui.add_space(12.0);
}

fn render_power_volume_row(
    ui: &mut Ui,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    w: f32,
) {
    let gap = 8.0;
    let power_w = (w * 0.32).clamp(100.0, 150.0);
    let vol_w = w - power_w - gap;
    let h = 40.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;

        // Power Button
        if custom_button(ui, vec2(power_w, h), "Power", BtnIcon::Power, None, 14.0, false) {
            ctrl.key(keys::POWER);
        }

        // Volume Card with interactive slider
        let (rect, resp) = ui.allocate_exact_size(vec2(vol_w, h), Sense::click_and_drag());
        let bg = if resp.hovered() { Color32::from_rgb(20, 31, 50) } else { CARD_BG };
        let painter = ui.painter();
        painter.rect_filled(rect, 14.0, bg);
        painter.rect_stroke(rect, 14.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);

        // Speaker Icon
        let spk_c = pos2(rect.min.x + 20.0, rect.center().y);
        svg_volume_speaker(ui, Rect::from_center_size(spk_c, vec2(15.0, 15.0)), ICON_SLATE);

        // Volume text & track
        let text_x = rect.min.x + 38.0;
        let (vol_text, frac) = match snap.volume {
            Some((lvl, max, muted)) => {
                let f = if max > 0 { (lvl as f32 / max as f32).clamp(0.0, 1.0) } else { 0.0 };
                let txt = if muted { "Muted".to_string() } else { format!("Volume {lvl}/{max}") };
                (txt, f)
            }
            None => {
                let pct = (state.volume_level * 100.0).round() as u32;
                (format!("Volume {pct}%"), state.volume_level)
            }
        };

        painter.text(
            pos2(text_x, rect.center().y - 8.0),
            Align2::LEFT_CENTER,
            &vol_text,
            FontId::proportional(11.0),
            TEXT_MUTED,
        );

        let track_start = pos2(text_x, rect.center().y + 8.0);
        let track_w = (rect.max.x - 16.0 - text_x).max(40.0);
        let track_rect = Rect::from_min_size(track_start, vec2(track_w, 3.5));
        painter.rect_filled(track_rect, 1.75, Color32::from_rgb(10, 16, 26));

        let filled_w = track_w * frac;
        if filled_w > 0.0 {
            let filled_rect = Rect::from_min_size(track_start, vec2(filled_w, 3.5));
            painter.rect_filled(filled_rect, 1.75, ACCENT);
        }

        let knob_x = track_start.x + filled_w;
        let knob_c = pos2(knob_x, track_start.y + 1.75);
        painter.circle_filled(knob_c, 5.0, ACCENT);

        if resp.dragged() || resp.clicked() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let new_f = ((pos.x - track_start.x) / track_w).clamp(0.0, 1.0);
                if (new_f - state.volume_level).abs() >= 0.04 {
                    haptics::trigger(HapticStyle::Tick);
                    if new_f > state.volume_level {
                        ctrl.key(keys::VOLUME_UP);
                    } else {
                        ctrl.key(keys::VOLUME_DOWN);
                    }
                    state.volume_level = new_f;
                }
            }
        }
    });
}

fn render_scalable_dpad(
    ui: &mut Ui,
    _state: &mut UiState,
    ctrl: &mut Controller,
    w: f32,
) {
    let dpad_total_size = (w * 0.48).clamp(160.0, 220.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, dpad_total_size), Sense::click_and_drag());
    let center = rect.center();

    let dist = 46.0 * (dpad_total_size / 158.0);
    let r_lobe = 23.0 * (dpad_total_size / 158.0);
    let _r_fillet = 16.0 * (dpad_total_size / 158.0);
    let center_r = 29.0 * (dpad_total_size / 158.0);
    let pad_r = 21.0 * (dpad_total_size / 158.0);

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

    let fire = crate::ui::common::widgets::hold_logic(ui, &resp);
    if fire {
        haptics::trigger(HapticStyle::Heavy);
        if center_pressed {
            ctrl.key(keys::DPAD_CENTER);
        } else if let Some(dir) = pressed_dir {
            match dir {
                Dir::Up => ctrl.key(keys::DPAD_UP),
                Dir::Down => ctrl.key(keys::DPAD_DOWN),
                Dir::Left => ctrl.key(keys::DPAD_LEFT),
                Dir::Right => ctrl.key(keys::DPAD_RIGHT),
            }
        }
    }

    draw_responsive_clover_dpad(
        ui,
        center,
        dpad_total_size,
        hovered_dir,
        pressed_dir,
        center_hovered,
        center_pressed,
    );
}

fn render_nav_row(ui: &mut Ui, ctrl: &mut Controller, w: f32) {
    let gap = (w * 0.022).clamp(6.0, 10.0);
    let btn_w = (w - 2.0 * gap) / 3.0;
    let h = (w * 0.10).clamp(34.0, 44.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(btn_w, h), "Back", BtnIcon::Back, None, 14.0, false) {
            ctrl.key(keys::BACK);
        }
        if custom_button(ui, vec2(btn_w, h), "Home", BtnIcon::Home, None, 14.0, false) {
            ctrl.key(keys::HOME);
        }
        if custom_button(ui, vec2(btn_w, h), "Menu", BtnIcon::Menu, None, 14.0, false) {
            ctrl.key(keys::MENU);
        }
    });
}

fn render_function_row_1(ui: &mut Ui, ctrl: &mut Controller, w: f32) {
    let gap = (w * 0.020).clamp(5.0, 8.0);
    let btn_w = (w - 3.0 * gap) / 4.0;
    let h = (w * 0.095).clamp(32.0, 42.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(btn_w, h), "Vol -", BtnIcon::VolMinus, None, 12.0, true) {
            ctrl.key(keys::VOLUME_DOWN);
        }
        if custom_button(ui, vec2(btn_w, h), "Mute", BtnIcon::Mute, None, 12.0, false) {
            ctrl.key(keys::VOLUME_MUTE);
        }
        if custom_button(ui, vec2(btn_w, h), "Vol +", BtnIcon::VolPlus, None, 12.0, true) {
            ctrl.key(keys::VOLUME_UP);
        }
        if custom_button(ui, vec2(btn_w, h), "Input", BtnIcon::Input, None, 12.0, false) {
            ctrl.key(keys::TV_INPUT);
        }
    });
}

fn render_function_row_2(ui: &mut Ui, ctrl: &mut Controller, w: f32) {
    let gap = (w * 0.020).clamp(5.0, 8.0);
    let btn_w = (w - 3.0 * gap) / 4.0;
    let h = (w * 0.095).clamp(32.0, 42.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(btn_w, h), "Ch -", BtnIcon::ChevronDown, None, 12.0, true) {
            ctrl.key(keys::CHANNEL_DOWN);
        }
        if custom_button(ui, vec2(btn_w, h), "Ch +", BtnIcon::ChevronUp, None, 12.0, true) {
            ctrl.key(keys::CHANNEL_UP);
        }
        if custom_button(ui, vec2(btn_w, h), "Guide", BtnIcon::Guide, None, 12.0, false) {
            ctrl.key(keys::GUIDE);
        }
        if custom_button(ui, vec2(btn_w, h), "Info", BtnIcon::Info, None, 12.0, false) {
            ctrl.key(keys::INFO);
        }
    });
}

fn render_function_row_3(ui: &mut Ui, ctrl: &mut Controller, w: f32) {
    let gap = (w * 0.022).clamp(6.0, 10.0);
    let btn_w = (w - 2.0 * gap) / 3.0;
    let h = (w * 0.10).clamp(34.0, 44.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button_stacked(ui, vec2(btn_w, h), "Prev", BtnIcon::Prev, None, 12.0, true) {
            ctrl.key(keys::MEDIA_PREVIOUS);
        }
        if custom_button_stacked(ui, vec2(btn_w, h), "Play/Pause", BtnIcon::PlayPause, None, 12.0, false) {
            ctrl.key(keys::MEDIA_PLAY_PAUSE);
        }
        if custom_button_stacked(ui, vec2(btn_w, h), "Next", BtnIcon::Next, None, 12.0, true) {
            ctrl.key(keys::MEDIA_NEXT);
        }
    });
}

fn render_function_row_4(ui: &mut Ui, ctrl: &mut Controller, w: f32) {
    let gap = (w * 0.022).clamp(6.0, 10.0);
    let btn_w = (w - 2.0 * gap) / 3.0;
    let h = (w * 0.095).clamp(32.0, 42.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(btn_w, h), "Search", BtnIcon::Search, None, 12.0, false) {
            ctrl.key(keys::SEARCH);
        }
        if custom_button(ui, vec2(btn_w, h), "Captions", BtnIcon::Captions, None, 12.0, false) {
            ctrl.key(keys::CAPTIONS);
        }
        if custom_button(ui, vec2(btn_w, h), "Settings", BtnIcon::Settings, None, 12.0, false) {
            ctrl.key(keys::SETTINGS);
        }
    });
}

fn render_apps_section(ui: &mut Ui, ctrl: &mut Controller, w: f32) {
    ui.label(eframe::egui::RichText::new("Apps").size(13.5).strong().color(TEXT_WHITE));
    ui.add_space(4.0);

    let gap = (w * 0.022).clamp(6.0, 10.0);
    let btn_w = (w - 2.0 * gap) / 3.0;
    let h = (w * 0.095).clamp(32.0, 42.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(btn_w, h), "YouTube", BtnIcon::YouTube, None, 12.0, false) {
            ctrl.launch("https://www.youtube.com");
        }
        if custom_button(ui, vec2(btn_w, h), "Netflix", BtnIcon::Netflix, None, 12.0, false) {
            ctrl.launch("https://www.netflix.com/title");
        }
        if custom_button(ui, vec2(btn_w, h), "Prime Video", BtnIcon::Prime, None, 12.0, false) {
            ctrl.launch("https://app.primevideo.com");
        }
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(btn_w, h), "Disney+", BtnIcon::Disney, None, 12.0, false) {
            ctrl.launch("https://www.disneyplus.com");
        }
        if custom_button(ui, vec2(btn_w, h), "Hotstar", BtnIcon::Hotstar, None, 12.0, false) {
            ctrl.launch("https://www.hotstar.com");
        }
        if custom_button(ui, vec2(btn_w, h), "Spotify", BtnIcon::Spotify, None, 12.0, false) {
            ctrl.launch("spotify://");
        }
    });
}
