//! Tab 2: Apps Launcher (Bento Grid) for Material 3 Expressive UI.

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::ui::state::{ActiveSheet, ActiveTab, UiState};
use crate::ui::theme::ThemePalette;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, StrokeKind, Ui};

pub struct AppCardData {
    pub name: &'static str,
    pub category: &'static str,
    pub media_sample: &'static str,
    pub badge_bg: Color32,
    pub badge_fg: Color32,
    pub url: &'static str,
}

pub fn render_expressive_apps_tab(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ctrl: &mut Controller,
    now: f64,
) {
    let avail_w = ui.available_width();

    // Top Header: Title & Find button
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(eframe::egui::RichText::new("Your TV Apps").size(16.0).strong().color(palette.text));
            ui.label(eframe::egui::RichText::new("Tap any app to launch immediately on TV").size(11.5).color(palette.text_muted));
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let find_btn_size = vec2(78.0, 32.0);
            let (find_rect, find_resp) = ui.allocate_exact_size(find_btn_size, Sense::click());
            let painter = ui.painter();
            let f_bg = if find_resp.hovered() { palette.surface_highest } else { palette.surface_high };
            painter.rect_filled(find_rect, 16.0, f_bg);
            painter.rect_stroke(find_rect, 16.0, palette.outline(), StrokeKind::Inside);

            let icon_rect = Rect::from_center_size(pos2(find_rect.min.x + 18.0, find_rect.center().y), vec2(14.0, 14.0));
            svg_search(ui, icon_rect, palette.primary);

            painter.text(
                pos2(find_rect.min.x + 32.0, find_rect.center().y),
                Align2::LEFT_CENTER,
                "Find",
                FontId::proportional(12.0),
                palette.text,
            );

            if find_resp.clicked() {
                haptics::trigger(HapticStyle::Light);
                state.toggle_sheet(ActiveSheet::Keyboard);
            }
        });
    });

    ui.add_space(10.0);

    // 2-Column Bento Grid
    let gap = 10.0;
    let card_w = (avail_w - gap) * 0.5;
    let card_h = 104.0;

    let apps = [
        AppCardData {
            name: "YouTube",
            category: "Video & Live",
            media_sample: "Red Bull Rampage 4K",
            badge_bg: Color32::from_rgba_unmultiplied(239, 68, 68, 45),
            badge_fg: Color32::from_rgb(248, 113, 113),
            url: "https://www.youtube.com",
        },
        AppCardData {
            name: "Netflix",
            category: "Currently Active",
            media_sample: "Stranger Things — S4:E9",
            badge_bg: Color32::from_rgba_unmultiplied(225, 29, 72, 45),
            badge_fg: Color32::from_rgb(251, 113, 133),
            url: "https://www.netflix.com/title",
        },
        AppCardData {
            name: "Prime Video",
            category: "Movies & Series",
            media_sample: "Fallout — S1:E4",
            badge_bg: Color32::from_rgba_unmultiplied(56, 189, 248, 45),
            badge_fg: Color32::from_rgb(125, 211, 252),
            url: "https://app.primevideo.com",
        },
        AppCardData {
            name: "Disney+ Hotstar",
            category: "Sports & Originals",
            media_sample: "Premier League Live",
            badge_bg: Color32::from_rgba_unmultiplied(129, 140, 248, 45),
            badge_fg: Color32::from_rgb(165, 180, 252),
            url: "https://www.hotstar.com",
        },
        AppCardData {
            name: "Spotify",
            category: "TV Audio Cast",
            media_sample: "Midnight Lo-Fi Beats",
            badge_bg: Color32::from_rgba_unmultiplied(52, 211, 153, 45),
            badge_fg: Color32::from_rgb(110, 231, 183),
            url: "spotify://",
        },
        AppCardData {
            name: "Apple TV+",
            category: "Originals",
            media_sample: "Severance — S2:E1",
            badge_bg: Color32::from_rgba_unmultiplied(148, 163, 184, 45),
            badge_fg: Color32::from_rgb(226, 232, 240),
            url: "https://tv.apple.com",
        },
    ];

    for chunk in apps.chunks(2) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for app in chunk {
                let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::click());
                let painter = ui.painter();

                let draw_rect = if resp.is_pointer_button_down_on() { rect.shrink(1.2) } else { rect };
                let bg = if resp.is_pointer_button_down_on() {
                    palette.surface_highest
                } else if resp.hovered() {
                    palette.surface_high
                } else {
                    palette.surface_low
                };

                painter.rect_filled(draw_rect, 26.0, bg);
                painter.rect_stroke(draw_rect, 26.0, palette.outline(), StrokeKind::Inside);

                // Badge top-left
                let badge_rect = Rect::from_min_size(draw_rect.min + vec2(14.0, 14.0), vec2(38.0, 38.0));
                painter.rect_filled(badge_rect, 14.0, app.badge_bg);

                // Badge icon
                let icon_box = Rect::from_center_size(badge_rect.center(), vec2(20.0, 20.0));
                match app.name {
                    "YouTube" => svg_youtube(ui, icon_box),
                    "Netflix" => {
                        painter.text(badge_rect.center(), Align2::CENTER_CENTER, "N", FontId::proportional(17.0), app.badge_fg);
                    }
                    "Prime Video" => svg_prime(ui, icon_box),
                    "Disney+ Hotstar" => svg_hotstar(ui, icon_box),
                    "Spotify" => svg_spotify(ui, icon_box),
                    _ => svg_tv(ui, icon_box, app.badge_fg),
                }

                // Title & Subtitle at bottom
                let text_x = draw_rect.min.x + 14.0;
                let text_y = draw_rect.max.y - 32.0;

                painter.text(
                    pos2(text_x, text_y),
                    Align2::LEFT_CENTER,
                    app.name,
                    FontId::proportional(13.5),
                    palette.text,
                );

                painter.text(
                    pos2(text_x, text_y + 15.0),
                    Align2::LEFT_CENTER,
                    app.category,
                    FontId::proportional(11.0),
                    palette.text_muted,
                );

                if resp.clicked() {
                    haptics::trigger(HapticStyle::Medium);
                    ctrl.launch(app.url);
                    state.active_app_name = app.name.to_string();
                    state.active_media_title = app.media_sample.to_string();
                    state.active_app_subtitle = "Streaming".to_string();
                    state.send_command_feedback(format!("Launching {}", app.name), now);
                    state.tab = ActiveTab::Remote;
                }
            }
        });
        ui.add_space(gap);
    }
}
