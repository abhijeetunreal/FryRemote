//! Advanced Mode Apps Tab: Full-screen streaming apps catalog.

use crate::controller::Controller;
use crate::ui::common::widgets::{custom_button, BtnIcon};
use eframe::egui::{vec2, RichText, Ui};

pub fn render_advanced_apps_tab(ui: &mut Ui, ctrl: &mut Controller, w: f32) {
    ui.heading(RichText::new("All TV Apps").color(crate::ui::advanced::remote_tab::TEXT_WHITE));
    ui.label(RichText::new("Quick launch any streaming service on your Google TV").color(crate::ui::advanced::remote_tab::TEXT_MUTED).size(12.0));
    ui.add_space(12.0);

    let apps: &[(&str, BtnIcon, &str)] = &[
        ("YouTube", BtnIcon::YouTube, "https://www.youtube.com"),
        ("Netflix", BtnIcon::Netflix, "https://www.netflix.com/title"),
        ("Prime", BtnIcon::Prime, "https://app.primevideo.com"),
        ("Disney+", BtnIcon::Disney, "https://www.disneyplus.com"),
        ("Hotstar", BtnIcon::Hotstar, "https://www.hotstar.com"),
        ("Spotify", BtnIcon::Spotify, "spotify://"),
        ("Apple TV", BtnIcon::None, "https://tv.apple.com"),
        ("Twitch", BtnIcon::None, "https://www.twitch.tv"),
        ("Plex", BtnIcon::None, "plex://"),
        ("Crunchyroll", BtnIcon::None, "https://www.crunchyroll.com"),
        ("Hulu", BtnIcon::None, "https://www.hulu.com"),
        ("Tubi", BtnIcon::None, "https://tubitv.com"),
    ];

    let gap = 8.0;
    let btn_w = (w - 2.0 * gap) / 3.0;
    let h = 54.0;

    for chunk in apps.chunks(3) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (name, icon, link) in chunk {
                if custom_button(ui, vec2(btn_w, h), name, *icon, None, 14.0, false) {
                    ctrl.launch(link);
                }
            }
        });
        ui.add_space(6.0);
    }
    ui.add_space(30.0);
}
