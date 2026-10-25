//! D-Pad and Touchpad interaction components for FryRemote.

use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::ui::common::widgets::{blend_color, hold_logic};
use crate::ui::state::UiState;
use crate::ui::theme::ThemePalette;
use eframe::egui::{
    pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DpadAction {
    Up,
    Down,
    Left,
    Right,
    Center,
}

// ---------------------------------------------------------------------------
// Material 3 Expressive Squircle D-Pad
// ---------------------------------------------------------------------------

pub fn draw_expressive_dpad(
    ui: &mut Ui,
    palette: &ThemePalette,
    _state: &mut UiState,
    size: f32,
) -> Option<DpadAction> {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    let painter = ui.painter();
    let center = rect.center();

    // Squircle container
    let cr = size * 0.27; // ~68px on 250px
    painter.rect_filled(rect, cr, palette.surface_low);
    painter.rect_stroke(rect, cr, palette.outline(), StrokeKind::Inside);

    // Subtle crosshair guides inside
    let inner_box = rect.shrink(size * 0.09);
    painter.rect_stroke(
        inner_box,
        cr * 0.72,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 12)),
        StrokeKind::Inside,
    );

    let center_btn_size = size * 0.38; // ~96px on 250px
    let center_rect = Rect::from_center_size(center, vec2(center_btn_size, center_btn_size));
    let center_cr = center_btn_size * 0.36;

    let pointer_pos = resp.interact_pointer_pos().or_else(|| ui.input(|i| i.pointer.hover_pos()));
    let is_down = resp.is_pointer_button_down_on();

    let mut hovered_dir = None;
    let mut center_hovered = false;

    if let Some(pos) = pointer_pos {
        if center_rect.contains(pos) {
            center_hovered = true;
        } else if rect.contains(pos) {
            let delta = pos - center;
            let angle = delta.y.atan2(delta.x);
            if angle >= -std::f32::consts::FRAC_PI_4 && angle < std::f32::consts::FRAC_PI_4 {
                hovered_dir = Some(DpadAction::Right);
            } else if angle >= std::f32::consts::FRAC_PI_4 && angle < 3.0 * std::f32::consts::FRAC_PI_4 {
                hovered_dir = Some(DpadAction::Down);
            } else if angle >= -3.0 * std::f32::consts::FRAC_PI_4 && angle < -std::f32::consts::FRAC_PI_4 {
                hovered_dir = Some(DpadAction::Up);
            } else {
                hovered_dir = Some(DpadAction::Left);
            }
        }
    }

    let pressed_dir = if is_down { hovered_dir } else { None };
    let center_pressed = is_down && center_hovered;

    // Directional button zones & visual arrow indicators
    let arrow_size = 28.0;
    let arrow_color = palette.text;
    let active_color = palette.primary;

    // Up
    let up_c = pos2(center.x, rect.min.y + size * 0.16);
    let _up_rect = Rect::from_center_size(up_c, vec2(arrow_size, arrow_size));
    if pressed_dir == Some(DpadAction::Up) {
        let pad_zone = Rect::from_min_max(pos2(rect.min.x + cr * 0.5, rect.min.y), pos2(rect.max.x - cr * 0.5, center_rect.min.y));
        painter.rect_filled(pad_zone, cr * 0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 14));
    }
    let up_stroke = if pressed_dir == Some(DpadAction::Up) { Stroke::new(2.4, active_color) } else { Stroke::new(2.0, arrow_color) };
    painter.add(Shape::line(
        vec![pos2(up_c.x - 9.0, up_c.y + 4.0), pos2(up_c.x, up_c.y - 5.0), pos2(up_c.x + 9.0, up_c.y + 4.0)],
        up_stroke,
    ));

    // Down
    let down_c = pos2(center.x, rect.max.y - size * 0.16);
    let _down_rect = Rect::from_center_size(down_c, vec2(arrow_size, arrow_size));
    if pressed_dir == Some(DpadAction::Down) {
        let pad_zone = Rect::from_min_max(pos2(rect.min.x + cr * 0.5, center_rect.max.y), pos2(rect.max.x - cr * 0.5, rect.max.y));
        painter.rect_filled(pad_zone, cr * 0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 14));
    }
    let down_stroke = if pressed_dir == Some(DpadAction::Down) { Stroke::new(2.4, active_color) } else { Stroke::new(2.0, arrow_color) };
    painter.add(Shape::line(
        vec![pos2(down_c.x - 9.0, down_c.y - 4.0), pos2(down_c.x, down_c.y + 5.0), pos2(down_c.x + 9.0, down_c.y - 4.0)],
        down_stroke,
    ));

    // Left
    let left_c = pos2(rect.min.x + size * 0.16, center.y);
    let _left_rect = Rect::from_center_size(left_c, vec2(arrow_size, arrow_size));
    if pressed_dir == Some(DpadAction::Left) {
        let pad_zone = Rect::from_min_max(pos2(rect.min.x, rect.min.y + cr * 0.5), pos2(center_rect.min.x, rect.max.y - cr * 0.5));
        painter.rect_filled(pad_zone, cr * 0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 14));
    }
    let left_stroke = if pressed_dir == Some(DpadAction::Left) { Stroke::new(2.4, active_color) } else { Stroke::new(2.0, arrow_color) };
    painter.add(Shape::line(
        vec![pos2(left_c.x + 4.0, left_c.y - 9.0), pos2(left_c.x - 5.0, left_c.y), pos2(left_c.x + 4.0, left_c.y + 9.0)],
        left_stroke,
    ));

    // Right
    let right_c = pos2(rect.max.x - size * 0.16, center.y);
    let _right_rect = Rect::from_center_size(right_c, vec2(arrow_size, arrow_size));
    if pressed_dir == Some(DpadAction::Right) {
        let pad_zone = Rect::from_min_max(pos2(center_rect.max.x, rect.min.y + cr * 0.5), pos2(rect.max.x, rect.max.y - cr * 0.5));
        painter.rect_filled(pad_zone, cr * 0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 14));
    }
    let right_stroke = if pressed_dir == Some(DpadAction::Right) { Stroke::new(2.4, active_color) } else { Stroke::new(2.0, arrow_color) };
    painter.add(Shape::line(
        vec![pos2(right_c.x - 4.0, right_c.y - 9.0), pos2(right_c.x + 5.0, right_c.y), pos2(right_c.x - 4.0, right_c.y + 9.0)],
        right_stroke,
    ));

    // Center "OK" squircle button
    let ok_bg = if center_pressed {
        blend_color(palette.primary, Color32::BLACK, 0.15)
    } else if center_hovered {
        blend_color(palette.primary, Color32::WHITE, 0.12)
    } else {
        palette.primary
    };

    let ok_draw_rect = if center_pressed { center_rect.shrink(3.0) } else { center_rect };
    let ok_cr = if center_pressed { center_cr * 0.8 } else { center_cr };

    painter.rect_filled(ok_draw_rect, ok_cr, ok_bg);
    painter.rect_stroke(
        ok_draw_rect,
        ok_cr,
        Stroke::new(1.2, Color32::from_rgba_unmultiplied(255, 255, 255, 50)),
        StrokeKind::Inside,
    );

    painter.text(
        ok_draw_rect.center(),
        Align2::CENTER_CENTER,
        "OK",
        FontId::proportional(15.0),
        palette.on_primary,
    );

    // Fire action with press-and-hold repeater
    let fire = hold_logic(ui, &resp);
    if fire {
        haptics::trigger(HapticStyle::Heavy);
        if center_pressed {
            return Some(DpadAction::Center);
        } else if let Some(dir) = pressed_dir {
            return Some(dir);
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Material 3 Expressive Gesture Swipe Pad
// ---------------------------------------------------------------------------

pub fn draw_swipe_touchpad(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    size: f32,
    _now: f64,
) -> Option<DpadAction> {
    let (rect, resp) = ui.allocate_exact_size(vec2(size, size), Sense::click_and_drag());
    let painter = ui.painter();
    let center = rect.center();

    let cr = size * 0.27;
    painter.rect_filled(rect, cr, palette.surface_low);
    painter.rect_stroke(rect, cr, palette.outline(), StrokeKind::Inside);

    // Center icon & instructions
    let icon_rect = Rect::from_center_size(center + vec2(0.0, -18.0), vec2(36.0, 36.0));
    draw_icon_touch_app(painter, icon_rect, palette.primary);

    painter.text(
        center + vec2(0.0, 16.0),
        Align2::CENTER_CENTER,
        "Swipe to navigate",
        FontId::proportional(13.0),
        palette.text,
    );
    painter.text(
        center + vec2(0.0, 32.0),
        Align2::CENTER_CENTER,
        "Tap anywhere to Select",
        FontId::proportional(11.0),
        palette.text_muted,
    );

    // Ripple feedback
    if let Some(pos) = resp.interact_pointer_pos() {
        if resp.is_pointer_button_down_on() {
            painter.circle_filled(pos, 22.0, Color32::from_rgba_unmultiplied(palette.primary.r(), palette.primary.g(), palette.primary.b(), 45));
            painter.circle_stroke(pos, 22.0, Stroke::new(1.5, palette.primary));
        }
    }

    // Gesture handling: Swipe vs Tap
    const STEP: f32 = 36.0;
    if resp.dragged() {
        state.touch_acc += resp.drag_delta();
        if state.touch_acc.x > STEP {
            haptics::trigger(HapticStyle::Tick);
            state.touch_acc.x = 0.0;
            return Some(DpadAction::Right);
        }
        if state.touch_acc.x < -STEP {
            haptics::trigger(HapticStyle::Tick);
            state.touch_acc.x = 0.0;
            return Some(DpadAction::Left);
        }
        if state.touch_acc.y > STEP {
            haptics::trigger(HapticStyle::Tick);
            state.touch_acc.y = 0.0;
            return Some(DpadAction::Down);
        }
        if state.touch_acc.y < -STEP {
            haptics::trigger(HapticStyle::Tick);
            state.touch_acc.y = 0.0;
            return Some(DpadAction::Up);
        }
    } else {
        state.touch_acc = Vec2::ZERO;
    }

    if resp.clicked() {
        haptics::trigger(HapticStyle::Medium);
        return Some(DpadAction::Center);
    }

    None
}

// ---------------------------------------------------------------------------
// Advanced Mode Responsive Clover D-Pad
// ---------------------------------------------------------------------------

pub fn draw_responsive_clover_dpad(
    ui: &mut Ui,
    center: Pos2,
    total_size: f32,
    hovered_dir: Option<Dir>,
    pressed_dir: Option<Dir>,
    center_hovered: bool,
    center_pressed: bool,
) {
    let scale = (total_size / 158.0).clamp(0.85, 1.6);
    let dist = 46.0 * scale;
    let r_lobe = 23.0 * scale;
    let r_fillet = 16.0 * scale;
    let _pad_radius = 21.0 * scale;
    let _center_r = 29.0 * scale;

    draw_clover_dpad(
        ui,
        center,
        dist,
        r_lobe,
        r_fillet,
        hovered_dir,
        pressed_dir,
        center_hovered,
        center_pressed,
    );
}
