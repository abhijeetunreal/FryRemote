//! Advanced Mode Touchpad Tab: Full-screen responsive gesture trackpad.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::keys;
use crate::ui::common::widgets::{custom_button, BtnIcon};
use crate::ui::state::UiState;
use eframe::egui::{vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

pub fn render_advanced_touchpad_tab(
    ui: &mut Ui,
    state: &mut UiState,
    ctrl: &mut Controller,
    w: f32,
    max_h: f32,
) {
    ui.heading(eframe::egui::RichText::new("Touchpad").color(Color32::from_rgb(248, 250, 252)));
    ui.label(eframe::egui::RichText::new("Swipe in any direction to move • Tap anywhere to select").color(Color32::from_rgb(100, 116, 139)).size(12.0));
    ui.add_space(8.0);

    // Expand to cover full available screen height!
    let pad_h = (max_h - 110.0).max(280.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, pad_h), Sense::click_and_drag());
    let painter = ui.painter();
    painter.rect_filled(rect, 24.0, Color32::from_rgb(15, 23, 42));
    painter.rect_stroke(rect, 24.0, Stroke::new(1.2, Color32::from_rgb(28, 43, 68)), StrokeKind::Inside);

    // Center icon & instruction
    let c = rect.center();
    svg_nav_touchpad(ui, Rect::from_center_size(c + vec2(0.0, -20.0), vec2(32.0, 32.0)), Color32::from_rgb(148, 163, 184));
    painter.text(
        c + vec2(0.0, 16.0),
        Align2::CENTER_CENTER,
        "Swipe to Navigate • Tap to Select",
        FontId::proportional(14.5),
        Color32::from_rgb(100, 116, 139),
    );

    // Gesture handling with tactile tick haptics
    const STEP: f32 = 40.0;
    if resp.dragged() {
        state.touch_acc += resp.drag_delta();
        while state.touch_acc.x > STEP {
            haptics::trigger(HapticStyle::Tick);
            ctrl.key(keys::DPAD_RIGHT);
            state.touch_acc.x -= STEP;
        }
        while state.touch_acc.x < -STEP {
            haptics::trigger(HapticStyle::Tick);
            ctrl.key(keys::DPAD_LEFT);
            state.touch_acc.x += STEP;
        }
        while state.touch_acc.y > STEP {
            haptics::trigger(HapticStyle::Tick);
            ctrl.key(keys::DPAD_DOWN);
            state.touch_acc.y -= STEP;
        }
        while state.touch_acc.y < -STEP {
            haptics::trigger(HapticStyle::Tick);
            ctrl.key(keys::DPAD_UP);
            state.touch_acc.y += STEP;
        }
    } else {
        state.touch_acc = Vec2::ZERO;
    }

    // Glowing neon ring under pointer during interaction
    if let Some(pos) = resp.interact_pointer_pos() {
        painter.circle_filled(pos, 28.0, Color32::from_rgba_premultiplied(59, 130, 246, 35));
        painter.circle_filled(pos, 10.0, Color32::from_rgba_premultiplied(56, 189, 248, 80));
        painter.circle_stroke(pos, 28.0, Stroke::new(1.5, Color32::from_rgb(59, 130, 246)));
    }

    if resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        ctrl.key(keys::DPAD_CENTER);
    }

    ui.add_space(12.0);

    // Quick Back and Home buttons under trackpad
    let gap = 8.0;
    let btn_w = (w - gap) / 2.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        if custom_button(ui, vec2(btn_w, 48.0), "Back", BtnIcon::Back, None, 14.0, false) {
            ctrl.key(keys::BACK);
        }
        if custom_button(ui, vec2(btn_w, 48.0), "Home", BtnIcon::Home, None, 14.0, false) {
            ctrl.key(keys::HOME);
        }
    });
}
