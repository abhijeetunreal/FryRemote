//! Iconic Circular Clickpad Assembly for Apple TV Edition.
//! Supports outer directional clicks (Up/Down/Left/Right) with repeat hold
//! and center concave surface for both Tap to Select and Drag/Swipe gestures.

use crate::haptics::{self, HapticStyle};
use crate::ui::common::widgets::hold_logic;
use crate::ui::state::UiState;
use crate::ui::theme::ThemePalette;
use eframe::egui::{vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClickpadAction {
    Up,
    Down,
    Left,
    Right,
    Center,
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
}

pub fn draw_apple_clickpad(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    pad_size: f32,
    now: f64,
) -> Option<ClickpadAction> {
    let (pad_rect, _) = ui.allocate_exact_size(vec2(pad_size, pad_size), Sense::hover());
    let center = pad_rect.center();
    let outer_r = pad_size * 0.5;
    let painter = ui.painter();

    let mut result = None;

    // 1. Outer Ring Base Surface
    painter.circle_filled(center, outer_r, palette.surface_low);
    painter.circle_stroke(center, outer_r, Stroke::new(1.2, palette.outline_color));

    // Outer tactile dots at 4 cardinal directions
    let dot_dist = outer_r * 0.84;
    let dot_r = 2.4;
    let dot_color = Color32::from_rgba_unmultiplied(255, 255, 255, 90);

    painter.circle_filled(center + vec2(0.0, -dot_dist), dot_r, dot_color);
    painter.circle_filled(center + vec2(0.0, dot_dist), dot_r, dot_color);
    painter.circle_filled(center + vec2(-dot_dist, 0.0), dot_r, dot_color);
    painter.circle_filled(center + vec2(dot_dist, 0.0), dot_r, dot_color);

    // Outer Directional Click Zones
    let btn_len = outer_r * 0.70;
    let btn_thick = outer_r * 0.40;

    // UP
    let up_rect = Rect::from_center_size(center + vec2(0.0, -dot_dist + 2.0), vec2(btn_len, btn_thick));
    let up_resp = ui.interact(up_rect, ui.scope_id().with("pad_up"), Sense::click());
    if up_resp.is_pointer_button_down_on() {
        painter.circle_filled(center + vec2(0.0, -dot_dist), dot_r + 2.0, palette.primary);
    }
    if hold_logic(ui, &up_resp) {
        haptics::trigger(HapticStyle::Tick);
        result = Some(ClickpadAction::Up);
    }

    // DOWN
    let down_rect = Rect::from_center_size(center + vec2(0.0, dot_dist - 2.0), vec2(btn_len, btn_thick));
    let down_resp = ui.interact(down_rect, ui.scope_id().with("pad_down"), Sense::click());
    if down_resp.is_pointer_button_down_on() {
        painter.circle_filled(center + vec2(0.0, dot_dist), dot_r + 2.0, palette.primary);
    }
    if hold_logic(ui, &down_resp) {
        haptics::trigger(HapticStyle::Tick);
        result = Some(ClickpadAction::Down);
    }

    // LEFT
    let left_rect = Rect::from_center_size(center + vec2(-dot_dist + 2.0, 0.0), vec2(btn_thick, btn_len));
    let left_resp = ui.interact(left_rect, ui.scope_id().with("pad_left"), Sense::click());
    if left_resp.is_pointer_button_down_on() {
        painter.circle_filled(center + vec2(-dot_dist, 0.0), dot_r + 2.0, palette.primary);
    }
    if hold_logic(ui, &left_resp) {
        haptics::trigger(HapticStyle::Tick);
        result = Some(ClickpadAction::Left);
    }

    // RIGHT
    let right_rect = Rect::from_center_size(center + vec2(dot_dist - 2.0, 0.0), vec2(btn_thick, btn_len));
    let right_resp = ui.interact(right_rect, ui.scope_id().with("pad_right"), Sense::click());
    if right_resp.is_pointer_button_down_on() {
        painter.circle_filled(center + vec2(dot_dist, 0.0), dot_r + 2.0, palette.primary);
    }
    if hold_logic(ui, &right_resp) {
        haptics::trigger(HapticStyle::Tick);
        result = Some(ClickpadAction::Right);
    }

    // 2. Center Concave Touch & Click Surface
    let center_r = outer_r * 0.61;
    let center_rect = Rect::from_center_size(center, vec2(center_r * 2.0, center_r * 2.0));
    let center_resp = ui.interact(center_rect, ui.scope_id().with("pad_center_touch"), Sense::click_and_drag());

    let c_draw_r = if center_resp.is_pointer_button_down_on() {
        center_r - 1.5
    } else {
        center_r
    };

    painter.circle_filled(center, c_draw_r, palette.secondary_container);
    painter.circle_stroke(
        center,
        c_draw_r,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 14)),
    );

    // Subtle concave label
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "CLICK / SWIPE",
        FontId::proportional((pad_size * 0.042).clamp(8.5, 12.0)),
        Color32::from_rgba_unmultiplied(palette.text.r(), palette.text.g(), palette.text.b(), 65),
    );

    // Ripple visual feedback
    if center_resp.drag_started() || center_resp.clicked() {
        if let Some(pos) = center_resp.interact_pointer_pos() {
            state.touch_ripples.push((pos, now));
            state.touch_acc = eframe::egui::Vec2::ZERO;
        }
    }

    // Clean up old ripples
    state.touch_ripples.retain(|(_, t)| now - *t < 0.35);
    for (pos, t) in &state.touch_ripples {
        let age = (now - *t) as f32;
        let frac = (age / 0.35).clamp(0.0, 1.0);
        let r = 8.0 + frac * 28.0;
        let alpha = ((1.0 - frac) * 90.0) as u8;
        painter.circle_filled(*pos, r, Color32::from_rgba_unmultiplied(palette.primary.r(), palette.primary.g(), palette.primary.b(), alpha));
    }

    // Gesture detection
    if center_resp.dragged() {
        state.touch_acc += center_resp.drag_delta();
    }

    if center_resp.drag_stopped() {
        let dx = state.touch_acc.x;
        let dy = state.touch_acc.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist >= 18.0 {
            haptics::trigger(HapticStyle::Medium);
            if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    result = Some(ClickpadAction::SwipeRight);
                } else {
                    result = Some(ClickpadAction::SwipeLeft);
                }
            } else {
                if dy > 0.0 {
                    result = Some(ClickpadAction::SwipeDown);
                } else {
                    result = Some(ClickpadAction::SwipeUp);
                }
            }
        }
        state.touch_acc = eframe::egui::Vec2::ZERO;
    } else if center_resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        result = Some(ClickpadAction::Center);
    }

    result
}
