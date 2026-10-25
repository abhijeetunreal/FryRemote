//! Material 3 Expressive UI (Primary UI) Implementation for FryRemote.

pub mod apps_tab;
pub mod header;
pub mod keypad_tab;
pub mod now_playing;
pub mod remote_tab;
pub mod sheets;

use crate::controller::Controller;
use crate::haptics::{self, HapticStyle};
use crate::icons::*;
use crate::state::Shared;
use crate::ui::state::{ActiveSheet, ActiveTab, UiState};
use crate::ui::theme::{ThemeKind, ThemePalette};
use crate::ui::UiMode;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

pub fn render_expressive_ui(
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
    let is_desktop_wide = total_rect.width() > 460.0;

    // Draw dark presentation background if in wide window, or theme background if compact
    if is_desktop_wide {
        ui.painter().rect_filled(total_rect, 0.0, Color32::from_rgb(7, 9, 12));
    } else {
        ui.painter().rect_filled(total_rect, 0.0, palette.bg);
    }

    let max_card_w = 410.0f32;
    let content_w = max_card_w.min(total_rect.width() - 16.0).max(280.0);
    let pad_x = ((total_rect.width() - content_w) * 0.5).max(0.0);

    // Top Desktop Presentation Bar (Theme circles, Mode Toggle, UX Notes)
    if is_desktop_wide {
        let bar_h = 34.0;
        let bar_rect = Rect::from_min_size(pos2(pad_x, total_rect.min.y + 6.0), vec2(content_w, bar_h));
        let mut bar_ui = ui.new_child(egui::UiBuilder::new().max_rect(bar_rect));

        bar_ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let (dot_rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 4.0, palette.primary);
            ui.label(
                eframe::egui::RichText::new("Material 3 Expressive UI")
                    .strong()
                    .size(12.0)
                    .color(Color32::from_rgb(226, 232, 240)),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                // UX Notes button
                if ui.button(eframe::egui::RichText::new("UX Notes").size(11.0)).clicked() {
                    haptics::trigger(HapticStyle::Light);
                    state.toggle_sheet(ActiveSheet::UxNotes);
                }

                // Mode switch button
                if ui
                    .button(
                        eframe::egui::RichText::new("🛠️ Advanced Mode")
                            .size(11.0)
                            .color(palette.primary),
                    )
                    .clicked()
                {
                    haptics::trigger(HapticStyle::Heavy);
                    *ui_mode = UiMode::Advanced;
                }

                // Theme selector dots
                let themes = [
                    ThemeKind::Sky,
                    ThemeKind::Violet,
                    ThemeKind::Emerald,
                    ThemeKind::Light,
                ];
                for t in themes {
                    let is_sel = *theme == t;
                    let (dot_rect, resp) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
                    ui.painter().circle_filled(dot_rect.center(), 7.0, t.dot_color());
                    if is_sel {
                        ui.painter().circle_stroke(
                            dot_rect.center(),
                            8.5,
                            Stroke::new(1.8, Color32::WHITE),
                        );
                    }
                    if resp.clicked() {
                        haptics::trigger(HapticStyle::Light);
                        *theme = t;
                        state.send_command_feedback(format!("Theme: {}", t.name()), now);
                    }
                }
            });
        });
    }

    // Phone Container Dimensions
    let (_frame_rect, container_rect) = if is_desktop_wide {
        let frame_top = total_rect.min.y + 44.0;
        let frame_h = (total_rect.height() - 54.0).min(840.0);
        let f_rect = Rect::from_min_size(pos2(pad_x, frame_top), vec2(content_w, frame_h));
        ui.painter().rect_filled(f_rect, 36.0, palette.bg);
        ui.painter().rect_stroke(
            f_rect,
            36.0,
            Stroke::new(3.0, Color32::from_rgb(30, 41, 59)),
            StrokeKind::Inside,
        );
        let inner = f_rect.shrink2(vec2(8.0, 10.0));
        (f_rect, inner)
    } else {
        let c_rect = Rect::from_min_size(
            pos2(pad_x, total_rect.min.y + 8.0),
            vec2(content_w, total_rect.height() - 16.0),
        );
        (total_rect, c_rect)
    };

    let inner_w = container_rect.width();

    // 1. Top Unified Header (Pinned to top of container)
    let header_h = 44.0;
    let header_rect = Rect::from_min_size(container_rect.min, vec2(inner_w, header_h));
    let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect));
    header::render_expressive_header(&mut header_ui, palette, state, ctrl, snap, theme, ui_mode, now);

    // 2. Contextual Dynamic Now Playing Bar (Pinned below header)
    let np_top = header_rect.max.y + 8.0;
    let np_h = 58.0;
    let np_rect = Rect::from_min_size(pos2(container_rect.min.x, np_top), vec2(inner_w, np_h));
    let mut np_ui = ui.new_child(egui::UiBuilder::new().max_rect(np_rect));
    now_playing::render_now_playing_bar(&mut np_ui, palette, state, ctrl, snap, now);

    // 3. Material 3 Expressive Bottom Navigation Bar (PINNED AT THE VERY BOTTOM OF CONTAINER)
    let nav_bar_h = 56.0;
    let nav_bar_rect = Rect::from_min_size(
        pos2(container_rect.min.x, container_rect.max.y - nav_bar_h),
        vec2(inner_w, nav_bar_h),
    );
    let mut nav_ui = ui.new_child(egui::UiBuilder::new().max_rect(nav_bar_rect));
    render_bottom_nav(&mut nav_ui, palette, state, ui_mode, now);

    // 4. Viewport (Sits in the space between Now Playing and Bottom Nav)
    let vp_top = np_rect.max.y + 8.0;
    let vp_bottom = nav_bar_rect.min.y - 8.0;
    let vp_rect = Rect::from_min_max(
        pos2(container_rect.min.x, vp_top),
        pos2(container_rect.max.x, vp_bottom.max(vp_top + 100.0)),
    );

    let mut vp_ui = ui.new_child(egui::UiBuilder::new().max_rect(vp_rect));
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut vp_ui, |ui| {
            match state.tab {
                ActiveTab::Remote => {
                    remote_tab::render_expressive_remote_tab(ui, palette, state, ctrl, snap, now);
                }
                ActiveTab::Apps => {
                    apps_tab::render_expressive_apps_tab(ui, palette, state, ctrl, now);
                }
                ActiveTab::Keypad => {
                    keypad_tab::render_expressive_keypad_tab(ui, palette, state, ctrl, now);
                }
            }
        });

    // 5. Bottom Sheets & Modals
    sheets::render_expressive_sheets(ui, palette, state, ctrl, snap, ui_mode, screen, now);
}

fn render_bottom_nav(
    ui: &mut Ui,
    palette: &ThemePalette,
    state: &mut UiState,
    ui_mode: &mut UiMode,
    now: f64,
) {
    let avail_w = ui.available_width();
    let bar_h = 56.0;

    let (nav_rect, _) = ui.allocate_exact_size(vec2(avail_w, bar_h), Sense::hover());
    let painter = ui.painter();

    painter.rect_filled(nav_rect, bar_h * 0.5, palette.surface_low);
    painter.rect_stroke(nav_rect, bar_h * 0.5, palette.outline(), StrokeKind::Inside);

    let tabs = [
        ("Remote", "remote"),
        ("Apps", "apps"),
        ("Keypad", "keypad"),
        ("Advanced", "advanced"),
    ];

    let tab_w = (nav_rect.width() - 8.0) / 4.0;

    for (i, &(label, icon_type)) in tabs.iter().enumerate() {
        let is_active = match label {
            "Remote" => state.tab == ActiveTab::Remote,
            "Apps" => state.tab == ActiveTab::Apps,
            "Keypad" => state.tab == ActiveTab::Keypad,
            _ => false,
        };

        let item_rect = Rect::from_min_size(
            pos2(nav_rect.min.x + 4.0 + (i as f32) * tab_w, nav_rect.min.y + 4.0),
            vec2(tab_w, bar_h - 8.0),
        );
        let resp = ui.interact(item_rect, ui.scope_id().with(label), Sense::click());

        if is_active {
            painter.rect_filled(item_rect, (bar_h - 8.0) * 0.5, palette.primary_container);
        }

        let fg = if is_active {
            palette.on_primary_container
        } else {
            palette.text_muted
        };

        let icon_box = Rect::from_center_size(
            pos2(item_rect.center().x - 14.0, item_rect.center().y),
            vec2(16.0, 16.0),
        );

        match icon_type {
            "remote" => svg_nav_remote(ui, icon_box, fg),
            "apps" => svg_nav_apps(ui, icon_box, fg),
            "keypad" => svg_dialpad(ui, icon_box, fg),
            "advanced" => svg_tune(ui, icon_box, fg),
            _ => {}
        }

        painter.text(
            pos2(item_rect.center().x + 5.0, item_rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(11.0),
            fg,
        );

        if resp.clicked() {
            match label {
                "Remote" => {
                    haptics::trigger(HapticStyle::Light);
                    state.tab = ActiveTab::Remote;
                }
                "Apps" => {
                    haptics::trigger(HapticStyle::Light);
                    state.tab = ActiveTab::Apps;
                }
                "Keypad" => {
                    haptics::trigger(HapticStyle::Light);
                    state.tab = ActiveTab::Keypad;
                }
                "Advanced" => {
                    haptics::trigger(HapticStyle::Heavy);
                    *ui_mode = UiMode::Advanced;
                    state.send_command_feedback("Advanced Mode Activated", now);
                }
                _ => {}
            }
        }
    }
}
