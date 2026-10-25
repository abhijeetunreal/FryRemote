//! Common widgets and drawing utilities shared across FryRemote UI modes.

use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::ui::theme::ThemePalette;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Hold Logic (Press and Hold Repeat)
// ---------------------------------------------------------------------------

pub fn hold_logic(ui: &Ui, resp: &Response) -> bool {
    let id = resp.id.with("hold");
    let now = ui.input(|i| i.time);
    if resp.is_pointer_button_down_on() {
        let next: Option<f64> = ui.data(|d| d.get_temp(id));
        ui.ctx().request_repaint();
        match next {
            None => {
                ui.data_mut(|d| d.insert_temp(id, now + 0.45));
                true
            }
            Some(t) if now >= t => {
                ui.data_mut(|d| d.insert_temp(id, now + 0.12));
                true
            }
            _ => false,
        }
    } else {
        ui.data_mut(|d| d.remove::<f64>(id));
        false
    }
}

// ---------------------------------------------------------------------------
// Live Status Indicator Dot (Pulsing Green / Amber / Red)
// ---------------------------------------------------------------------------

pub fn draw_live_status_dot(painter: &egui::Painter, center: egui::Pos2, active: bool, standby: bool, time: f64) {
    if active && !standby {
        // Pulsing outer ripple
        let pulse = ((time * 2.2).sin() * 0.5 + 0.5) as f32;
        let r_outer = 4.5 + pulse * 3.5;
        let alpha = ((1.0 - pulse) * 160.0) as u8;
        painter.circle_filled(center, r_outer, Color32::from_rgba_unmultiplied(74, 222, 128, alpha));
        // Inner solid dot
        painter.circle_filled(center, 4.0, Color32::from_rgb(74, 222, 128)); // #4ade80
    } else if standby {
        painter.circle_filled(center, 4.0, Color32::from_rgb(251, 191, 36)); // #fbbf24 amber
    } else {
        painter.circle_filled(center, 4.0, Color32::from_rgb(248, 113, 113)); // #f87171 red
    }
}

// ---------------------------------------------------------------------------
// Material 3 Expressive Morphing Button
// ---------------------------------------------------------------------------

pub struct M3Button<'a> {
    label: Option<&'a str>,
    icon: Option<BtnIcon>,
    size: Vec2,
    bg: Option<Color32>,
    text_color: Option<Color32>,
    corner_radius: f32,
    active_corner_radius: f32,
    stroke: Option<Stroke>,
    repeat: bool,
}

impl<'a> M3Button<'a> {
    pub fn new(size: Vec2) -> Self {
        Self {
            label: None,
            icon: None,
            size,
            bg: None,
            text_color: None,
            corner_radius: 20.0,
            active_corner_radius: 14.0,
            stroke: None,
            repeat: false,
        }
    }

    pub fn label(mut self, text: &'a str) -> Self {
        self.label = Some(text);
        self
    }

    pub fn icon(mut self, icon: BtnIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn bg(mut self, color: Color32) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn corner_radius(mut self, cr: f32) -> Self {
        self.corner_radius = cr;
        self
    }

    pub fn active_corner_radius(mut self, cr: f32) -> Self {
        self.active_corner_radius = cr;
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn repeat(mut self, rep: bool) -> Self {
        self.repeat = rep;
        self
    }

    pub fn show(self, ui: &mut Ui, palette: &ThemePalette) -> (bool, Response) {
        let (rect, resp) = ui.allocate_exact_size(self.size, Sense::click());
        let fire = if self.repeat {
            hold_logic(ui, &resp)
        } else {
            resp.clicked()
        };

        let is_down = resp.is_pointer_button_down_on();
        let is_hovered = resp.hovered();

        let base_bg = self.bg.unwrap_or(palette.surface_low);
        let bg = if is_down {
            blend_color(base_bg, palette.primary, 0.22)
        } else if is_hovered {
            blend_color(base_bg, palette.primary, 0.10)
        } else {
            base_bg
        };

        let cr = if is_down { self.active_corner_radius } else { self.corner_radius };
        let draw_rect = if is_down { rect.shrink(1.0) } else { rect };

        let stroke = self.stroke.unwrap_or(palette.outline());
        let stroke = if is_down {
            Stroke::new(1.6, palette.primary)
        } else if is_hovered {
            Stroke::new(1.2, blend_color(palette.outline_color, palette.primary, 0.35))
        } else {
            stroke
        };

        let painter = ui.painter();
        painter.rect_filled(draw_rect, cr, bg);
        painter.rect_stroke(draw_rect, cr, stroke, StrokeKind::Inside);

        let fg_color = self.text_color.unwrap_or(palette.text);
        let c = draw_rect.center();

        match (self.label, self.icon) {
            (Some(text), Some(icon)) if icon != BtnIcon::None => {
                // Responsive horizontal icon + text
                let font_size = (self.size.y * 0.25).clamp(11.0, 14.5);
                let font_id = FontId::proportional(font_size);
                let text_layout = painter.layout_no_wrap(text.to_string(), font_id.clone(), fg_color);
                let text_w = text_layout.size().x;
                let icon_w = (self.size.y * 0.34).clamp(14.0, 20.0);
                let gap = (self.size.y * 0.10).clamp(4.0, 8.0);
                let total_w = icon_w + gap + text_w;
                let start_x = c.x - total_w * 0.5;

                let icon_rect = Rect::from_center_size(pos2(start_x + icon_w * 0.5, c.y), vec2(icon_w, icon_w));
                render_btn_svg(ui, icon_rect, icon, fg_color);

                painter.text(pos2(start_x + icon_w + gap, c.y), Align2::LEFT_CENTER, text, font_id, fg_color);
            }
            (Some(text), _) => {
                let font_size = (self.size.y * 0.28).clamp(11.0, 14.0);
                painter.text(c, Align2::CENTER_CENTER, text, FontId::proportional(font_size), fg_color);
            }
            (None, Some(icon)) => {
                let icon_dim = (self.size.y * 0.46).min(self.size.x * 0.46).clamp(14.0, 26.0);
                let icon_rect = Rect::from_center_size(c, vec2(icon_dim, icon_dim));
                render_btn_svg(ui, icon_rect, icon, fg_color);
            }
            (None, None) => {}
        }

        if fire {
            let haptic_style = match self.icon {
                Some(BtnIcon::Power) => HapticStyle::Heavy,
                Some(BtnIcon::None) | None => HapticStyle::Light,
                _ => HapticStyle::Medium,
            };
            haptics::trigger(haptic_style);
        }

        (fire, resp)
    }
}

// ---------------------------------------------------------------------------
// Legacy & Advanced Mode Buttons
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BtnIcon {
    None,
    Power,
    Tv,
    ChevronDown,
    ChevronUp,
    Gear,
    Back,
    Home,
    Menu,
    VolMinus,
    VolPlus,
    Mute,
    Input,
    Guide,
    Info,
    Prev,
    PlayPause,
    Next,
    Search,
    Captions,
    Settings,
    YouTube,
    Netflix,
    Prime,
    Disney,
    Hotstar,
    Spotify,
    AppleTv,
    Mic,
    Keyboard,
    Palette,
    Tune,
    Replay10,
    Forward10,
    Movie,
    Gamepad,
    TouchApp,
    Dialpad,
    Check,
    Return,
    ClosedCaptions,
    SpeakerGroup,
}

pub fn custom_button(
    ui: &mut Ui,
    size: Vec2,
    label: &str,
    icon: BtnIcon,
    custom_fill: Option<Color32>,
    corner_radius: f32,
    repeat: bool,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fire = if repeat {
        hold_logic(ui, &resp)
    } else {
        resp.clicked()
    };

    let is_down = resp.is_pointer_button_down_on();
    let bg = if is_down {
        custom_fill.unwrap_or(Color32::from_rgb(32, 52, 86))
    } else if resp.hovered() {
        custom_fill.unwrap_or(Color32::from_rgb(26, 42, 68))
    } else {
        custom_fill.unwrap_or(Color32::from_rgb(17, 28, 46))
    };

    let draw_rect = if is_down { rect.shrink(1.2) } else { rect };

    let stroke = if is_down {
        Stroke::new(1.8, Color32::from_rgb(56, 189, 248))
    } else if resp.hovered() {
        Stroke::new(1.0, Color32::from_rgb(45, 68, 108))
    } else {
        Stroke::new(1.0, Color32::from_rgb(28, 43, 68))
    };

    let painter = ui.painter();
    painter.rect_filled(draw_rect, corner_radius, bg);
    painter.rect_stroke(draw_rect, corner_radius, stroke, StrokeKind::Inside);

    let c = draw_rect.center();
    let text_color = if custom_fill.is_some() {
        Color32::WHITE
    } else {
        Color32::from_rgb(226, 232, 240)
    };

    let font_size = if size.x < 75.0 {
        11.5
    } else if size.y < 42.0 {
        12.0
    } else {
        13.0
    };
    let font_id = FontId::proportional(font_size);

    if icon == BtnIcon::None {
        painter.text(c, Align2::CENTER_CENTER, label, font_id, text_color);
    } else if label.is_empty() {
        let icon_box = Rect::from_center_size(c, vec2(17.0, 17.0));
        render_btn_svg(ui, icon_box, icon, text_color);
    } else {
        let text_layout = painter.layout_no_wrap(label.to_string(), font_id.clone(), text_color);
        let text_w = text_layout.size().x;
        let icon_w = 16.0;
        let gap = 5.0;
        let total_w = icon_w + gap + text_w;
        let start_x = c.x - total_w * 0.5;

        let icon_rect = Rect::from_center_size(pos2(start_x + icon_w * 0.5, c.y), vec2(icon_w, icon_w));
        render_btn_svg(ui, icon_rect, icon, text_color);

        let text_p = pos2(start_x + icon_w + gap, c.y);
        painter.text(text_p, Align2::LEFT_CENTER, label, font_id, text_color);
    }

    if fire {
        let style = match icon {
            BtnIcon::Power => HapticStyle::Heavy,
            BtnIcon::None => HapticStyle::Light,
            _ => HapticStyle::Medium,
        };
        haptics::trigger(style);
    }

    fire
}

pub fn custom_button_stacked(
    ui: &mut Ui,
    size: Vec2,
    label: &str,
    icon: BtnIcon,
    custom_fill: Option<Color32>,
    corner_radius: f32,
    repeat: bool,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fire = if repeat {
        hold_logic(ui, &resp)
    } else {
        resp.clicked()
    };

    let is_down = resp.is_pointer_button_down_on();
    let bg = if is_down {
        custom_fill.unwrap_or(Color32::from_rgb(32, 52, 86))
    } else if resp.hovered() {
        custom_fill.unwrap_or(Color32::from_rgb(26, 42, 68))
    } else {
        custom_fill.unwrap_or(Color32::from_rgb(17, 28, 46))
    };

    let draw_rect = if is_down { rect.shrink(1.2) } else { rect };

    let stroke = if is_down {
        Stroke::new(1.8, Color32::from_rgb(56, 189, 248))
    } else if resp.hovered() {
        Stroke::new(1.0, Color32::from_rgb(45, 68, 108))
    } else {
        Stroke::new(1.0, Color32::from_rgb(28, 43, 68))
    };

    let painter = ui.painter();
    painter.rect_filled(draw_rect, corner_radius, bg);
    painter.rect_stroke(draw_rect, corner_radius, stroke, StrokeKind::Inside);

    let c = draw_rect.center();
    let text_color = if custom_fill.is_some() {
        Color32::WHITE
    } else {
        Color32::from_rgb(226, 232, 240)
    };

    let icon_rect = Rect::from_center_size(pos2(c.x, c.y - 7.0), vec2(16.0, 13.0));
    render_btn_svg(ui, icon_rect, icon, text_color);

    let font_id = FontId::proportional(10.0);
    painter.text(pos2(c.x, c.y + 7.5), Align2::CENTER_CENTER, label, font_id, text_color);

    if fire {
        let style = match icon {
            BtnIcon::PlayPause => HapticStyle::Heavy,
            _ => HapticStyle::Medium,
        };
        haptics::trigger(style);
    }

    fire
}

pub fn render_btn_svg(ui: &Ui, rect: Rect, icon: BtnIcon, color: Color32) {
    let painter = ui.painter();
    match icon {
        BtnIcon::None => {}
        BtnIcon::Power => svg_power(ui, rect),
        BtnIcon::Tv => svg_tv(ui, rect, color),
        BtnIcon::ChevronDown => svg_chevron_down(ui, rect, color),
        BtnIcon::ChevronUp => svg_chevron_up(ui, rect, color),
        BtnIcon::Gear | BtnIcon::Settings => svg_settings(ui, rect, color),
        BtnIcon::Back => svg_back(ui, rect, color),
        BtnIcon::Home => svg_home(ui, rect, color),
        BtnIcon::Menu => svg_menu(ui, rect, color),
        BtnIcon::VolMinus => svg_vol_down(ui, rect, color),
        BtnIcon::VolPlus => svg_vol_up(ui, rect, color),
        BtnIcon::Mute => svg_vol_mute(ui, rect),
        BtnIcon::Input => svg_input(ui, rect, color),
        BtnIcon::Guide => svg_guide(ui, rect, color),
        BtnIcon::Info => svg_info(ui, rect, color),
        BtnIcon::Prev => svg_prev(ui, rect, color),
        BtnIcon::PlayPause => svg_play_pause(ui, rect, color),
        BtnIcon::Next => svg_next(ui, rect, color),
        BtnIcon::Search => svg_search(ui, rect, color),
        BtnIcon::Captions => svg_captions(ui, rect, color),
        BtnIcon::YouTube => svg_youtube(ui, rect),
        BtnIcon::Netflix => svg_netflix(ui, rect),
        BtnIcon::Prime => svg_prime(ui, rect),
        BtnIcon::Disney => svg_disney(ui, rect),
        BtnIcon::Hotstar => svg_hotstar(ui, rect),
        BtnIcon::Spotify => svg_spotify(ui, rect),
        BtnIcon::AppleTv => svg_tv(ui, rect, color),
        BtnIcon::Mic => svg_mic(ui, rect, color),
        BtnIcon::Keyboard => svg_keyboard(ui, rect, color),
        BtnIcon::Palette => svg_palette(ui, rect, color),
        BtnIcon::Tune => svg_tune(ui, rect, color),
        BtnIcon::Replay10 => svg_replay_10(ui, rect, color),
        BtnIcon::Forward10 => svg_forward_10(ui, rect, color),
        BtnIcon::Movie => svg_movie(ui, rect, color),
        BtnIcon::Gamepad => svg_gamepad(ui, rect, color),
        BtnIcon::TouchApp => svg_touch_app(ui, rect, color),
        BtnIcon::Dialpad => svg_dialpad(ui, rect, color),
        BtnIcon::Check => draw_icon_check(painter, rect, color),
        BtnIcon::Return => draw_icon_return(painter, rect, color),
        BtnIcon::ClosedCaptions => draw_icon_closed_captions(painter, rect, color),
        BtnIcon::SpeakerGroup => draw_icon_speaker_group(painter, rect, color),
    }
}

pub fn keypad(ui: &mut Ui, text: &mut String, hex: bool, max_len: usize) {
    let rows: &[&str] = if hex {
        &["0123", "4567", "89AB", "CDEF"]
    } else {
        &["123", "456", "789", ".0"]
    };
    let gap = 6.0;
    let cols = if hex { 4.0 } else { 3.0 };
    let w = ((ui.available_width() - gap * (cols - 1.0)) / cols).min(84.0);
    for row in rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for c in row.chars() {
                if custom_button(ui, vec2(w, 48.0), &c.to_string(), BtnIcon::None, None, 10.0, false)
                    && text.len() < max_len
                {
                    text.push(c);
                }
            }
            if !hex && *row == ".0" && custom_button(ui, vec2(w, 48.0), "Del", BtnIcon::None, None, 10.0, false) {
                text.pop();
            }
        });
        ui.add_space(4.0);
    }
    if hex {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            if custom_button(ui, vec2(w * 2.0 + gap, 44.0), "Del", BtnIcon::None, None, 10.0, false) {
                text.pop();
            }
            if custom_button(ui, vec2(w * 2.0 + gap, 44.0), "Clear", BtnIcon::None, None, 10.0, false) {
                text.clear();
            }
        });
    }
}

// Color blending helper
pub fn blend_color(base: Color32, overlay: Color32, factor: f32) -> Color32 {
    let f = factor.clamp(0.0, 1.0);
    let inv = 1.0 - f;
    Color32::from_rgba_premultiplied(
        ((base.r() as f32 * inv) + (overlay.r() as f32 * f)) as u8,
        ((base.g() as f32 * inv) + (overlay.g() as f32 * f)) as u8,
        ((base.b() as f32 * inv) + (overlay.b() as f32 * f)) as u8,
        base.a(),
    )
}
