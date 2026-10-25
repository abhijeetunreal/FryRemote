//! Advanced Mode More Tab: Virtual keyboard, tools & settings, haptics, diagnostics.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::keys;
use crate::state::{ConnStatus, Shared};
use crate::ui::common::widgets::{custom_button, BtnIcon};
use crate::ui::state::{AdvancedTab, UiState};
use eframe::egui::{self, vec2, Color32, RichText, Stroke, Ui};

pub fn render_advanced_more_tab(
    ui: &mut Ui,
    state: &mut UiState,
    ctrl: &mut Controller,
    snap: &Shared,
    w: f32,
    screen: &mut crate::app::Screen,
) {
    ui.heading(RichText::new("Tools & Settings").color(Color32::from_rgb(248, 250, 252)));
    ui.add_space(8.0);

    // TV Switcher Button
    if custom_button(ui, vec2(w, 46.0), "Switch / Manage TVs", BtnIcon::Tv, None, 14.0, false) {
        state.show_tv_selector = true;
        state.advanced_tab = AdvancedTab::Remote;
    }

    ui.add_space(12.0);

    // Direct Text Input Bar
    ui.label(RichText::new("Type on TV").size(15.0).strong().color(Color32::from_rgb(248, 250, 252)));
    ui.label(RichText::new("Type or paste search queries and text directly into the TV").color(Color32::from_rgb(100, 116, 139)).size(11.5));
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let te = egui::TextEdit::singleline(&mut state.typed_text)
            .hint_text("Type text here...")
            .desired_width(w - 74.0);
        let te_resp = ui.add(te);
        let enter_hit = te_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if ui.button("Send").clicked() || enter_hit {
            if !state.typed_text.is_empty() {
                ctrl.send_text(&state.typed_text);
                state.typed_text.clear();
            }
        }
    });

    ui.add_space(12.0);

    // Feedback & Vibration Settings Card
    egui::Frame::new()
        .fill(Color32::from_rgb(15, 23, 42))
        .corner_radius(14)
        .stroke(Stroke::new(1.0, Color32::from_rgb(28, 43, 68)))
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(w - 24.0);
            ui.label(RichText::new("Feedback & Vibration").strong().color(Color32::from_rgb(248, 250, 252)).size(14.5));
            ui.label(RichText::new("Calibrated tactile haptics & native system click sounds").color(Color32::from_rgb(100, 116, 139)).size(11.5));
            ui.add_space(8.0);

            let btn_w = (w - 24.0 - 24.0 - 8.0) / 2.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let vib_active = haptics::is_enabled();
                let vib_txt = if vib_active { "Vibration: ON" } else { "Vibration: OFF" };
                let vib_bg = if vib_active { Some(Color32::from_rgb(18, 52, 40)) } else { None };
                if custom_button(ui, vec2(btn_w, 36.0), vib_txt, BtnIcon::None, vib_bg, 10.0, false) {
                    let next = !vib_active;
                    haptics::set_enabled(next);
                    if next {
                        haptics::trigger(HapticStyle::Heavy);
                    }
                }

                let snd_active = haptics::is_sound_enabled();
                let snd_txt = if snd_active { "Sound: ON" } else { "Sound: OFF" };
                let snd_bg = if snd_active { Some(Color32::from_rgb(18, 52, 40)) } else { None };
                if custom_button(ui, vec2(btn_w, 36.0), snd_txt, BtnIcon::None, snd_bg, 10.0, false) {
                    let next = !snd_active;
                    haptics::set_sound_enabled(next);
                    if next {
                        haptics::trigger(HapticStyle::Light);
                    }
                }
            });

            ui.add_space(8.0);
            let test_w = w - 48.0;
            if custom_button(ui, vec2(test_w, 36.0), "⚡ Test Tactile Vibration", BtnIcon::None, Some(Color32::from_rgb(26, 42, 68)), 10.0, false) {
                haptics::trigger(HapticStyle::Heavy);
            }
        });

    ui.add_space(12.0);

    // Virtual Keyboard
    ui.label(RichText::new("Virtual Keypad").size(14.5).strong().color(Color32::from_rgb(248, 250, 252)));
    ui.add_space(6.0);

    let gap = 4.0;
    let kw = ((w - 9.0 * gap) / 10.0).floor();
    for row in ["1234567890", "qwertyuiop", "asdfghjkl", "zxcvbnm"] {
        let width = row.len() as f32 * kw + (row.len() as f32 - 1.0) * gap;
        let pad = ((w - width) / 2.0).max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(pad);
            ui.spacing_mut().item_spacing.x = gap;
            for c in row.chars() {
                if custom_button(ui, vec2(kw, 38.0), &c.to_string(), BtnIcon::None, None, 8.0, false) {
                    if let Some(k) = keys::key_for_char(c) {
                        ctrl.key(k);
                    }
                }
            }
        });
        ui.add_space(4.0);
    }

    // Punctuation and Space / Delete / Enter
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        let small = vec2(kw * 1.4, 40.0);
        for (label, c) in [(".", '.'), ("@", '@'), ("-", '-'), ("/", '/')] {
            if custom_button(ui, small, label, BtnIcon::None, None, 8.0, false) {
                if let Some(k) = keys::key_for_char(c) {
                    ctrl.key(k);
                }
            }
        }
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(w * 0.40, 42.0), "Space", BtnIcon::None, None, 10.0, false) {
            ctrl.key(keys::SPACE);
        }
        if custom_button(ui, vec2(w * 0.26, 42.0), "Delete", BtnIcon::None, None, 10.0, true) {
            ctrl.key(keys::DEL);
        }
        if custom_button(ui, vec2(w * 0.28, 42.0), "Enter", BtnIcon::None, Some(Color32::from_rgb(59, 130, 246)), 10.0, false) {
            ctrl.key(keys::ENTER);
        }
    });

    ui.add_space(16.0);

    // Connection Diagnostics
    egui::Frame::new()
        .fill(Color32::from_rgb(15, 23, 42))
        .corner_radius(14)
        .stroke(Stroke::new(1.0, Color32::from_rgb(28, 43, 68)))
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(w - 24.0);
            ui.label(RichText::new("Connection Info").strong().color(Color32::from_rgb(248, 250, 252)));
            let status_text = match &snap.status {
                ConnStatus::Connected => "Connected to TV",
                ConnStatus::Connecting => "Connecting...",
                ConnStatus::Failed(e) => e.as_str(),
                ConnStatus::Idle => "Idle",
            };
            ui.label(RichText::new(format!("Status: {status_text}")).color(Color32::from_rgb(100, 116, 139)));

            if let Some(tv) = &ctrl.active {
                ui.label(RichText::new(format!("TV: {} ({})", tv.name, tv.host)).color(Color32::from_rgb(100, 116, 139)));
            }

            ui.horizontal(|ui| {
                if ui.button("Disconnect").clicked() {
                    ctrl.disconnect();
                }
                if let Some(tv) = ctrl.active.clone() {
                    if ui.button("Re-Pair").clicked() {
                        ctrl.disconnect();
                        state.code_input.clear();
                        ctrl.start_pairing(&tv);
                        *screen = crate::app::Screen::Pair(tv);
                    }
                }
            });
        });

    ui.add_space(30.0);
}
