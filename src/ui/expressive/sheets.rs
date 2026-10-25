//! Bottom Sheets & Overlays for Material 3 Expressive UI.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::state::{Shared, Tv};
use crate::ui::common::widgets::keypad;
use crate::ui::state::{ActiveSheet, UiState};
use crate::ui::theme::ThemePalette;
use crate::ui::UiMode;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, StrokeKind, Ui};

pub fn render_expressive_sheets(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    ui_mode: &mut UiMode,
    screen: &mut crate::app::Screen,
    now: f64,
) {
    if state.active_sheet == ActiveSheet::None {
        return;
    }

    let screen_rect = ui.max_rect();
    let painter = ui.painter();

    // 1. Semi-transparent backdrop overlay
    let backdrop_resp = ui.interact(screen_rect, ui.scope_id().with("sheet_backdrop"), Sense::click());
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(160));

    if backdrop_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.close_sheet();
        return;
    }

    // 2. Render the specific active sheet / modal
    match state.active_sheet {
        ActiveSheet::Options => {
            render_options_sheet(ui, palette, state, ctrl, screen_rect, ui_mode, now);
        }
        ActiveSheet::Device => {
            render_device_sheet(ui, palette, state, ctrl, snap, screen_rect, screen, now);
        }
        ActiveSheet::Keyboard => {
            render_keyboard_sheet(ui, palette, state, ctrl, screen_rect, now);
        }
        ActiveSheet::MediaScrubber => {
            render_media_scrubber_sheet(ui, palette, state, ctrl, screen_rect, now);
        }
        ActiveSheet::Voice => {
            render_voice_overlay(ui, palette, state, ctrl, screen_rect, now);
        }
        ActiveSheet::UxNotes => {
            render_ux_modal(ui, palette, state, screen_rect);
        }
        ActiveSheet::Apps => {}
        ActiveSheet::None => {}
    }
}

fn sheet_rect_for(screen_rect: Rect, sheet_h: f32) -> Rect {
    let total_w = screen_rect.width();
    let is_wide = total_w > 500.0;
    let sheet_w = if is_wide {
        (total_w * 0.42).clamp(380.0, 480.0)
    } else {
        total_w
    };
    let sheet_x = screen_rect.min.x + (total_w - sheet_w) * 0.5;
    Rect::from_min_size(pos2(sheet_x, screen_rect.max.y - sheet_h), vec2(sheet_w, sheet_h))
}

// ---------------------------------------------------------------------------
// Sheet 1: TV Quick Controls
// ---------------------------------------------------------------------------

fn render_options_sheet(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    screen_rect: Rect,
    ui_mode: &mut UiMode,
    now: f64,
) {
    let sheet_h = 350.0;
    let sheet_rect = sheet_rect_for(screen_rect, sheet_h);
    let painter = ui.painter();
    painter.rect_filled(sheet_rect, 34.0, palette.surface);
    painter.rect_stroke(sheet_rect, 34.0, palette.outline(), StrokeKind::Inside);

    // Drag handle pill
    let handle_c = pos2(sheet_rect.center().x, sheet_rect.min.y + 12.0);
    painter.rect_filled(Rect::from_center_size(handle_c, vec2(40.0, 4.5)), 2.5, Color32::from_rgba_unmultiplied(255, 255, 255, 50));

    // Header
    let head_y = sheet_rect.min.y + 36.0;
    painter.text(
        pos2(sheet_rect.min.x + 20.0, head_y),
        Align2::LEFT_CENTER,
        "TV Quick Controls",
        FontId::proportional(16.0),
        palette.text,
    );

    // Mode Switcher Pill in Top Right
    let mode_pill_w = 110.0;
    let mode_rect = Rect::from_min_size(pos2(sheet_rect.max.x - 20.0 - mode_pill_w, head_y - 12.0), vec2(mode_pill_w, 24.0));
    let mode_resp = ui.interact(mode_rect, ui.scope_id().with("opt_mode_switch"), Sense::click());
    painter.rect_filled(mode_rect, 12.0, palette.surface_high);
    painter.rect_stroke(mode_rect, 12.0, palette.outline(), StrokeKind::Inside);
    painter.text(mode_rect.center(), Align2::CENTER_CENTER, "Advanced Mode", FontId::proportional(10.5), palette.primary);

    if mode_resp.clicked() {
        haptics::trigger(HapticStyle::Heavy);
        *ui_mode = UiMode::Advanced;
        state.close_sheet();
        return;
    }

    // Grid of Actions (3 columns x 2 rows)
    let grid_top = head_y + 24.0;
    let gap = 10.0;
    let card_w = (sheet_rect.width() - 40.0 - 2.0 * gap) / 3.0;
    let card_h = 72.0;

    let actions = [
        ("Input", "input"),
        ("Subtitles", "cc"),
        ("Guide", "guide"),
        ("Info", "info"),
        ("Audio Out", "audio"),
        ("TV Settings", "settings"),
    ];

    for (i, &(label, act)) in actions.iter().enumerate() {
        let col = (i % 3) as f32;
        let row = (i / 3) as f32;
        let x = sheet_rect.min.x + 20.0 + col * (card_w + gap);
        let y = grid_top + row * (card_h + gap);
        let btn_rect = Rect::from_min_size(pos2(x, y), vec2(card_w, card_h));
        let resp = ui.interact(btn_rect, ui.scope_id().with(act), Sense::click());

        let bg = if resp.hovered() { palette.surface_highest } else { palette.surface_low };
        painter.rect_filled(btn_rect, 20.0, bg);
        painter.rect_stroke(btn_rect, 20.0, palette.outline(), StrokeKind::Inside);

        let icon_c = pos2(btn_rect.center().x, btn_rect.min.y + 26.0);
        let icon_box = Rect::from_center_size(icon_c, vec2(22.0, 22.0));
        match act {
            "input" => svg_input(ui, icon_box, palette.primary),
            "cc" => draw_icon_closed_captions(painter, icon_box, palette.primary),
            "guide" => svg_guide(ui, icon_box, palette.primary),
            "info" => svg_info(ui, icon_box, palette.primary),
            "audio" => draw_icon_speaker_group(painter, icon_box, palette.primary),
            "settings" => svg_settings(ui, icon_box, palette.primary),
            _ => {}
        }

        painter.text(
            pos2(btn_rect.center().x, btn_rect.max.y - 18.0),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.5),
            palette.text,
        );

        if resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            match act {
                "input" => { ctrl.key(keys::TV_INPUT); state.send_command_feedback("Input Source", now); }
                "cc" => { ctrl.key(keys::CAPTIONS); state.send_command_feedback("Closed Captions", now); }
                "guide" => { ctrl.key(keys::GUIDE); state.send_command_feedback("TV Guide", now); }
                "info" => { ctrl.key(keys::INFO); state.send_command_feedback("Program Info", now); }
                "audio" => { state.send_command_feedback("Audio Output", now); }
                "settings" => { ctrl.key(keys::SETTINGS); state.send_command_feedback("TV Settings", now); }
                _ => {}
            }
            state.close_sheet();
        }
    }

    // Bottom close button
    let close_rect = Rect::from_center_size(pos2(sheet_rect.center().x, sheet_rect.max.y - 32.0), vec2(sheet_rect.width() - 40.0, 42.0));
    let close_resp = ui.interact(close_rect, ui.scope_id().with("opt_close"), Sense::click());
    painter.rect_filled(close_rect, 21.0, palette.surface_low);
    painter.rect_stroke(close_rect, 21.0, palette.outline(), StrokeKind::Inside);
    painter.text(close_rect.center(), Align2::CENTER_CENTER, "Done", FontId::proportional(13.0), palette.text);
    if close_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.close_sheet();
    }
}

// ---------------------------------------------------------------------------
// Sheet 2: Device Switcher
// ---------------------------------------------------------------------------

fn render_device_sheet(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    _snap: &Shared,
    screen_rect: Rect,
    screen: &mut crate::app::Screen,
    now: f64,
) {
    let sheet_h = 420.0;
    let sheet_rect = sheet_rect_for(screen_rect, sheet_h);
    let painter = ui.painter();
    painter.rect_filled(sheet_rect, 34.0, palette.surface);
    painter.rect_stroke(sheet_rect, 34.0, palette.outline(), StrokeKind::Inside);

    // Handle
    let handle_c = pos2(sheet_rect.center().x, sheet_rect.min.y + 12.0);
    painter.rect_filled(Rect::from_center_size(handle_c, vec2(40.0, 4.5)), 2.5, Color32::from_rgba_unmultiplied(255, 255, 255, 50));

    // Title
    painter.text(
        pos2(sheet_rect.min.x + 20.0, sheet_rect.min.y + 36.0),
        Align2::LEFT_CENTER,
        "Select Google TV",
        FontId::proportional(16.0),
        palette.text,
    );

    let mut item_y = sheet_rect.min.y + 60.0;
    let item_h = 56.0;
    let item_w = sheet_rect.width() - 40.0;

    // List of Saved TVs
    for tv in ctrl.store.tvs.clone() {
        let is_active = ctrl.active.as_ref().map(|t| t.host == tv.host).unwrap_or(false);
        let rect = Rect::from_min_size(pos2(sheet_rect.min.x + 20.0, item_y), vec2(item_w, item_h));
        let resp = ui.interact(rect, ui.scope_id().with(&tv.host), Sense::click());

        let bg = if is_active {
            palette.primary_container
        } else if resp.hovered() {
            palette.surface_high
        } else {
            palette.surface_low
        };

        let fg = if is_active { palette.on_primary_container } else { palette.text };
        painter.rect_filled(rect, 18.0, bg);
        painter.rect_stroke(rect, 18.0, palette.outline(), StrokeKind::Inside);

        // TV icon
        let icon_box = Rect::from_center_size(pos2(rect.min.x + 24.0, rect.center().y), vec2(20.0, 20.0));
        svg_tv(ui, icon_box, fg);

        // Labels
        painter.text(
            pos2(rect.min.x + 48.0, rect.center().y - 8.0),
            Align2::LEFT_CENTER,
            &tv.name,
            FontId::proportional(13.0),
            fg,
        );
        let sub = format!("{} • {}", tv.host, if is_active { "Connected" } else { "Saved" });
        painter.text(
            pos2(rect.min.x + 48.0, rect.center().y + 8.5),
            Align2::LEFT_CENTER,
            sub,
            FontId::proportional(11.0),
            if is_active { palette.on_primary_container } else { palette.text_muted },
        );

        if is_active {
            let check_box = Rect::from_center_size(pos2(rect.max.x - 24.0, rect.center().y), vec2(18.0, 18.0));
            draw_icon_check(painter, check_box, fg);
        }

        if resp.clicked() {
            haptics::trigger(HapticStyle::Light);
            if !is_active {
                ctrl.connect(&tv);
                state.send_command_feedback(format!("Switched to {}", tv.name), now);
            }
            state.close_sheet();
        }

        item_y += item_h + 8.0;
    }

    // Discovered or Add TV by IP
    let add_rect = Rect::from_min_size(pos2(sheet_rect.min.x + 20.0, item_y), vec2(item_w, 44.0));
    let add_resp = ui.interact(add_rect, ui.scope_id().with("add_tv_btn"), Sense::click());
    painter.rect_filled(add_rect, 16.0, palette.surface_low);
    painter.rect_stroke(add_rect, 16.0, palette.outline(), StrokeKind::Inside);
    painter.text(
        add_rect.center(),
        Align2::CENTER_CENTER,
        "+ Add TV by IP Address",
        FontId::proportional(12.5),
        palette.primary,
    );

    if add_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.show_ip_pad = !state.show_ip_pad;
    }

    if state.show_ip_pad {
        item_y += 50.0;
        let pad_zone = Rect::from_min_size(pos2(sheet_rect.min.x + 20.0, item_y), vec2(item_w, 140.0));
        ui.allocate_ui(pad_zone.size(), |ui| {
            ui.label(eframe::egui::RichText::new(if state.ip_input.is_empty() { "192.168.1._" } else { &state.ip_input }).monospace().color(palette.primary));
            keypad(ui, &mut state.ip_input, false, 15);
            let valid = state.ip_input.parse::<std::net::Ipv4Addr>().is_ok();
            if ui.add_enabled(valid, egui::Button::new("Pair & Connect").fill(palette.primary)).clicked() {
                let host = state.ip_input.clone();
                let new_tv = Tv { name: format!("Google TV ({host})"), host, paired: false };
                ctrl.store.upsert(new_tv.clone());
                ctrl.start_pairing(&new_tv);
                *screen = crate::app::Screen::Pair(new_tv);
                state.ip_input.clear();
                state.show_ip_pad = false;
                state.close_sheet();
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Sheet 3: TV Keyboard Input
// ---------------------------------------------------------------------------

fn render_keyboard_sheet(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    screen_rect: Rect,
    now: f64,
) {
    let sheet_h = 240.0;
    let sheet_rect = sheet_rect_for(screen_rect, sheet_h);
    let painter = ui.painter();
    painter.rect_filled(sheet_rect, 34.0, palette.surface);
    painter.rect_stroke(sheet_rect, 34.0, palette.outline(), StrokeKind::Inside);

    // Handle
    let handle_c = pos2(sheet_rect.center().x, sheet_rect.min.y + 12.0);
    painter.rect_filled(Rect::from_center_size(handle_c, vec2(40.0, 4.5)), 2.5, Color32::from_rgba_unmultiplied(255, 255, 255, 50));

    // Title & subtitle
    painter.text(
        pos2(sheet_rect.min.x + 20.0, sheet_rect.min.y + 36.0),
        Align2::LEFT_CENTER,
        "Type to TV",
        FontId::proportional(16.0),
        palette.text,
    );
    painter.text(
        pos2(sheet_rect.min.x + 20.0, sheet_rect.min.y + 54.0),
        Align2::LEFT_CENTER,
        "Skip tedious D-pad typing—type directly on your phone.",
        FontId::proportional(11.5),
        palette.text_muted,
    );

    // Form: Input box + Send button
    let form_y = sheet_rect.min.y + 80.0;
    let input_w = sheet_rect.width() - 40.0 - 74.0;
    let input_h = 46.0;

    let input_zone = Rect::from_min_size(pos2(sheet_rect.min.x + 20.0, form_y), vec2(sheet_rect.width() - 40.0, input_h));
    ui.allocate_ui(input_zone.size(), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let te = egui::TextEdit::singleline(&mut state.keyboard_input)
                .hint_text("Search movies, shows, or YouTube...")
                .desired_width(input_w);
            let te_resp = ui.add(te);

            let enter_hit = te_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let send_btn = egui::Button::new(eframe::egui::RichText::new("Send").strong().color(palette.on_primary))
                .fill(palette.primary)
                .corner_radius(16.0)
                .min_size(vec2(66.0, input_h));

            if ui.add(send_btn).clicked() || enter_hit {
                if !state.keyboard_input.is_empty() {
                    haptics::trigger(HapticStyle::Medium);
                    ctrl.send_text(&state.keyboard_input);
                    state.send_command_feedback(format!("Typed \"{}\"", state.keyboard_input), now);
                    state.keyboard_input.clear();
                    state.close_sheet();
                }
            }
        });
    });
}

// ---------------------------------------------------------------------------
// Sheet 4: Expanded Media Scrubber
// ---------------------------------------------------------------------------

fn render_media_scrubber_sheet(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    screen_rect: Rect,
    now: f64,
) {
    let sheet_h = 280.0;
    let sheet_rect = sheet_rect_for(screen_rect, sheet_h);
    ui.painter().rect_filled(sheet_rect, 34.0, palette.surface);
    ui.painter().rect_stroke(sheet_rect, 34.0, palette.outline(), StrokeKind::Inside);

    // Handle
    let handle_c = pos2(sheet_rect.center().x, sheet_rect.min.y + 12.0);
    ui.painter().rect_filled(Rect::from_center_size(handle_c, vec2(40.0, 4.5)), 2.5, Color32::from_rgba_unmultiplied(255, 255, 255, 50));

    // Media Badge & Titles
    let badge_rect = Rect::from_min_size(pos2(sheet_rect.min.x + 20.0, sheet_rect.min.y + 32.0), vec2(46.0, 46.0));
    ui.painter().rect_filled(badge_rect, 16.0, palette.primary_container);
    draw_icon_movie(ui.painter(), Rect::from_center_size(badge_rect.center(), vec2(24.0, 24.0)), palette.on_primary_container);

    let text_x = badge_rect.max.x + 12.0;
    ui.painter().text(
        pos2(text_x, badge_rect.center().y - 10.0),
        Align2::LEFT_CENTER,
        &state.active_app_name.to_uppercase(),
        FontId::proportional(11.0),
        palette.primary,
    );
    ui.painter().text(
        pos2(text_x, badge_rect.center().y + 10.0),
        Align2::LEFT_CENTER,
        &state.active_media_title,
        FontId::proportional(14.0),
        palette.text,
    );

    // Scrubber Slider
    let slider_y = sheet_rect.min.y + 106.0;
    let slider_zone = Rect::from_min_size(pos2(sheet_rect.min.x + 20.0, slider_y), vec2(sheet_rect.width() - 40.0, 36.0));

    ui.allocate_ui(slider_zone.size(), |ui| {
        let slider = egui::Slider::new(&mut state.media_scrubber_pos, 0.0..=100.0)
            .show_value(false)
            .trailing_fill(true);
        if ui.add(slider).dragged() {
            state.send_command_feedback("Scrubbing Timeline", now);
        }
    });

    // Time indicators
    ui.painter().text(pos2(sheet_rect.min.x + 20.0, slider_y + 30.0), Align2::LEFT_CENTER, "34:12", FontId::proportional(11.0), palette.text_muted);
    ui.painter().text(pos2(sheet_rect.max.x - 20.0, slider_y + 30.0), Align2::RIGHT_CENTER, "1:28:40", FontId::proportional(11.0), palette.text_muted);

    // Transport buttons: Prev, Play/Pause, Next
    let trans_y = sheet_rect.min.y + 195.0;
    let center_x = sheet_rect.center().x;

    // Prev
    let prev_rect = Rect::from_center_size(pos2(center_x - 70.0, trans_y), vec2(46.0, 46.0));
    let prev_resp = ui.interact(prev_rect, ui.scope_id().with("scrub_prev"), Sense::click());
    ui.painter().rect_filled(prev_rect, 23.0, palette.surface_low);
    svg_prev(ui, Rect::from_center_size(prev_rect.center(), vec2(18.0, 18.0)), palette.text);
    if prev_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        ctrl.key(keys::MEDIA_PREVIOUS);
        state.send_command_feedback("Previous Track", now);
    }

    // Play/Pause Large
    let pp_rect = Rect::from_center_size(pos2(center_x, trans_y), vec2(58.0, 58.0));
    let pp_resp = ui.interact(pp_rect, ui.scope_id().with("scrub_pp"), Sense::click());
    ui.painter().rect_filled(pp_rect, 22.0, palette.primary);

    if state.is_playing {
        let bar_w = 4.5;
        let bar_h = 16.0;
        ui.painter().rect_filled(Rect::from_center_size(pp_rect.center() + vec2(-5.0, 0.0), vec2(bar_w, bar_h)), 1.5, palette.on_primary);
        ui.painter().rect_filled(Rect::from_center_size(pp_rect.center() + vec2(5.0, 0.0), vec2(bar_w, bar_h)), 1.5, palette.on_primary);
    } else {
        let pts = vec![pp_rect.center() + vec2(-6.0, -9.0), pp_rect.center() + vec2(9.0, 0.0), pp_rect.center() + vec2(-6.0, 9.0)];
        ui.painter().add(eframe::egui::Shape::convex_polygon(pts, palette.on_primary, eframe::egui::Stroke::NONE));
    }
    if pp_resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        state.is_playing = !state.is_playing;
        ctrl.key(keys::MEDIA_PLAY_PAUSE);
    }

    // Next
    let next_rect = Rect::from_center_size(pos2(center_x + 70.0, trans_y), vec2(46.0, 46.0));
    let next_resp = ui.interact(next_rect, ui.scope_id().with("scrub_next"), Sense::click());
    ui.painter().rect_filled(next_rect, 23.0, palette.surface_low);
    svg_next(ui, Rect::from_center_size(next_rect.center(), vec2(18.0, 18.0)), palette.text);
    if next_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        ctrl.key(keys::MEDIA_NEXT);
        state.send_command_feedback("Next Track", now);
    }
}

// ---------------------------------------------------------------------------
// Voice Assistant Fullscreen Overlay
// ---------------------------------------------------------------------------

fn render_voice_overlay(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    _ctrl: &mut Controller,
    screen_rect: Rect,
    now: f64,
) {
    let painter = ui.painter();
    painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(210));

    let c = screen_rect.center() + vec2(0.0, -40.0);

    // Pulsing outer ripple
    let pulse = ((now * 2.8).sin() * 0.5 + 0.5) as f32;
    let r_pulse = 45.0 + pulse * 28.0;
    let alpha = ((1.0 - pulse) * 120.0) as u8;
    painter.circle_filled(c, r_pulse, Color32::from_rgba_unmultiplied(palette.primary.r(), palette.primary.g(), palette.primary.b(), alpha));

    // Center mic circle
    let r_mic = 40.0;
    painter.circle_filled(c, r_mic, palette.primary);
    draw_icon_mic(painter, Rect::from_center_size(c, vec2(34.0, 34.0)), palette.on_primary);

    // Text
    painter.text(
        c + vec2(0.0, 68.0),
        Align2::CENTER_CENTER,
        "Listening for Google TV...",
        FontId::proportional(17.0),
        Color32::WHITE,
    );
    painter.text(
        c + vec2(0.0, 94.0),
        Align2::CENTER_CENTER,
        "\"Play Stranger Things on Netflix\" or \"Turn volume to 40%\"",
        FontId::proportional(12.0),
        Color32::from_rgb(203, 213, 225),
    );

    // Tap to close pill
    let close_c = pos2(c.x, screen_rect.max.y - 70.0);
    let close_rect = Rect::from_center_size(close_c, vec2(180.0, 36.0));
    painter.rect_filled(close_rect, 18.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30));
    painter.text(close_c, Align2::CENTER_CENTER, "Tap anywhere to close", FontId::proportional(12.0), Color32::WHITE);

    // Tap anywhere to close
    let tap_resp = ui.interact(screen_rect, ui.scope_id().with("voice_close_tap"), Sense::click());
    if tap_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.close_sheet();
    }
}

// ---------------------------------------------------------------------------
// UX Notes Modal
// ---------------------------------------------------------------------------

fn render_ux_modal(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    screen_rect: Rect,
) {
    let modal_w = (screen_rect.width() - 40.0).clamp(280.0, 380.0);
    let modal_h = 440.0;
    let modal_rect = Rect::from_center_size(screen_rect.center(), vec2(modal_w, modal_h));
    let painter = ui.painter();

    painter.rect_filled(modal_rect, 28.0, palette.surface);
    painter.rect_stroke(modal_rect, 28.0, palette.outline(), StrokeKind::Inside);

    // Title & Close Button
    painter.text(
        pos2(modal_rect.min.x + 20.0, modal_rect.min.y + 28.0),
        Align2::LEFT_CENTER,
        "How Cognitive Load Was Reduced",
        FontId::proportional(14.5),
        palette.primary,
    );

    let close_c = pos2(modal_rect.max.x - 28.0, modal_rect.min.y + 28.0);
    let close_rect = Rect::from_center_size(close_c, vec2(28.0, 28.0));
    let close_resp = ui.interact(close_rect, ui.scope_id().with("ux_close_x"), Sense::click());
    painter.circle_filled(close_c, 14.0, palette.surface_high);
    painter.text(close_c, Align2::CENTER_CENTER, "✕", FontId::proportional(12.0), palette.text);
    if close_resp.clicked() {
        haptics::trigger(HapticStyle::Light);
        state.close_sheet();
    }

    // Scrollable / formatted notes
    let notes = [
        ("1. Tactile Vertical Rockers", "Volume and Channel use vertical pill rockers for intuitive muscle memory without looking."),
        ("2. Header & Status Consolidation", "Merged brand, device selection, and live connection dot into one unified pill."),
        ("3. Eliminates Screen Clutter", "Integrates D-Pad & Swipe mode into the squircle pad, eliminating redundant scrolling."),
        ("4. Contextual Now Playing Card", "Quick transport and scrubber right at the top, leaving full screen for essential remote controls."),
    ];

    let mut y = modal_rect.min.y + 60.0;
    for (heading, body) in notes {
        let card = Rect::from_min_size(pos2(modal_rect.min.x + 16.0, y), vec2(modal_w - 32.0, 72.0));
        painter.rect_filled(card, 16.0, palette.surface_low);
        painter.rect_stroke(card, 16.0, palette.outline(), StrokeKind::Inside);

        painter.text(pos2(card.min.x + 12.0, card.min.y + 18.0), Align2::LEFT_CENTER, heading, FontId::proportional(12.0), palette.text);
        painter.text(pos2(card.min.x + 12.0, card.min.y + 42.0), Align2::LEFT_CENTER, body, FontId::proportional(10.5), palette.text_muted);
        y += 82.0;
    }
}
