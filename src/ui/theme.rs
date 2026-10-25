//! Material 3 Expressive & Classic Theme Definitions for FryRemote.

use eframe::egui::{self, Color32, Stroke};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeKind {
    Sky,      // Material Sky (Google TV Blue) - Default
    Violet,   // Expressive Violet
    Emerald,  // Mint Emerald
    Light,    // Daylight Soft
    Classic,  // Deep Slate Navy (Classic Advanced Mode)
}

impl ThemeKind {
    pub fn next(&self) -> Self {
        match self {
            ThemeKind::Sky => ThemeKind::Violet,
            ThemeKind::Violet => ThemeKind::Emerald,
            ThemeKind::Emerald => ThemeKind::Light,
            ThemeKind::Light => ThemeKind::Sky,
            ThemeKind::Classic => ThemeKind::Sky,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeKind::Sky => "Sky Blue",
            ThemeKind::Violet => "Expressive Violet",
            ThemeKind::Emerald => "Mint Emerald",
            ThemeKind::Light => "Daylight Soft",
            ThemeKind::Classic => "Deep Slate Classic",
        }
    }

    pub fn dot_color(&self) -> Color32 {
        match self {
            ThemeKind::Sky => Color32::from_rgb(144, 205, 255),
            ThemeKind::Violet => Color32::from_rgb(211, 188, 253),
            ThemeKind::Emerald => Color32::from_rgb(130, 217, 180),
            ThemeKind::Light => Color32::from_rgb(244, 247, 251),
            ThemeKind::Classic => Color32::from_rgb(59, 130, 246),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ThemePalette {
    pub kind: ThemeKind,
    pub is_dark: bool,
    pub bg: Color32,
    pub surface: Color32,
    pub surface_low: Color32,
    pub surface_high: Color32,
    pub surface_highest: Color32,
    pub primary: Color32,
    pub on_primary: Color32,
    pub primary_container: Color32,
    pub on_primary_container: Color32,
    pub secondary_container: Color32,
    pub on_secondary_container: Color32,
    pub tertiary: Color32,
    pub tertiary_container: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub outline_color: Color32,
}

impl ThemePalette {
    pub fn for_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Sky => Self::sky(),
            ThemeKind::Violet => Self::violet(),
            ThemeKind::Emerald => Self::emerald(),
            ThemeKind::Light => Self::light(),
            ThemeKind::Classic => Self::classic(),
        }
    }

    pub fn sky() -> Self {
        Self {
            kind: ThemeKind::Sky,
            is_dark: true,
            bg: Color32::from_rgb(14, 19, 24),              // #0e1318
            surface: Color32::from_rgb(20, 26, 33),         // #141a21
            surface_low: Color32::from_rgb(25, 32, 40),     // #192028
            surface_high: Color32::from_rgb(34, 43, 53),    // #222b35
            surface_highest: Color32::from_rgb(44, 55, 68), // #2c3744
            primary: Color32::from_rgb(144, 205, 255),      // #90cdff
            on_primary: Color32::from_rgb(0, 52, 82),       // #003452
            primary_container: Color32::from_rgb(0, 75, 115),// #004b73
            on_primary_container: Color32::from_rgb(203, 230, 255), // #cbe6ff
            secondary_container: Color32::from_rgb(41, 56, 70),     // #293846
            on_secondary_container: Color32::from_rgb(205, 229, 255),// #cde5ff
            tertiary: Color32::from_rgb(227, 181, 255),     // #e3b5ff
            tertiary_container: Color32::from_rgb(88, 44, 117), // #582c75
            text: Color32::from_rgb(227, 233, 240),         // #e3e9f0
            text_muted: Color32::from_rgb(142, 154, 168),   // #8e9aa8
            outline_color: Color32::from_rgba_unmultiplied(255, 255, 255, 22),
        }
    }

    pub fn violet() -> Self {
        Self {
            kind: ThemeKind::Violet,
            is_dark: true,
            bg: Color32::from_rgb(18, 16, 23),              // #121017
            surface: Color32::from_rgb(25, 22, 32),         // #191620
            surface_low: Color32::from_rgb(32, 28, 41),     // #201c29
            surface_high: Color32::from_rgb(42, 37, 54),    // #2a2536
            surface_highest: Color32::from_rgb(54, 47, 69), // #362f45
            primary: Color32::from_rgb(211, 188, 253),      // #d3bcfd
            on_primary: Color32::from_rgb(56, 38, 92),      // #38265c
            primary_container: Color32::from_rgb(79, 61, 116),// #4f3d74
            on_primary_container: Color32::from_rgb(235, 221, 255), // #ebddff
            secondary_container: Color32::from_rgb(51, 45, 65),     // #332d41
            on_secondary_container: Color32::from_rgb(232, 222, 248),// #e8def8
            tertiary: Color32::from_rgb(244, 183, 209),     // #f4b7d1
            tertiary_container: Color32::from_rgb(99, 59, 78), // #633b4e
            text: Color32::from_rgb(234, 228, 242),         // #eae4f2
            text_muted: Color32::from_rgb(152, 143, 166),   // #988fa6
            outline_color: Color32::from_rgba_unmultiplied(255, 255, 255, 22),
        }
    }

    pub fn emerald() -> Self {
        Self {
            kind: ThemeKind::Emerald,
            is_dark: true,
            bg: Color32::from_rgb(13, 20, 17),              // #0d1411
            surface: Color32::from_rgb(19, 28, 24),         // #131c18
            surface_low: Color32::from_rgb(24, 35, 30),     // #18231e
            surface_high: Color32::from_rgb(33, 47, 41),    // #212f29
            surface_highest: Color32::from_rgb(43, 60, 53), // #2b3c35
            primary: Color32::from_rgb(130, 217, 180),      // #82d9b4
            on_primary: Color32::from_rgb(0, 56, 38),       // #003826
            primary_container: Color32::from_rgb(0, 81, 57),// #005139
            on_primary_container: Color32::from_rgb(158, 246, 207), // #9ef6cf
            secondary_container: Color32::from_rgb(38, 57, 49),     // #263931
            on_secondary_container: Color32::from_rgb(207, 233, 221),// #cfe9dd
            tertiary: Color32::from_rgb(166, 204, 224),     // #a6cce0
            tertiary_container: Color32::from_rgb(36, 75, 92), // #244b5c
            text: Color32::from_rgb(225, 234, 229),         // #e1eae5
            text_muted: Color32::from_rgb(138, 155, 146),   // #8a9b92
            outline_color: Color32::from_rgba_unmultiplied(255, 255, 255, 22),
        }
    }

    pub fn light() -> Self {
        Self {
            kind: ThemeKind::Light,
            is_dark: false,
            bg: Color32::from_rgb(244, 247, 251),           // #f4f7fb
            surface: Color32::from_rgb(255, 255, 255),      // #ffffff
            surface_low: Color32::from_rgb(238, 243, 248),  // #eef3f8
            surface_high: Color32::from_rgb(227, 235, 243), // #e3ebf3
            surface_highest: Color32::from_rgb(214, 225, 236),// #d6e1ec
            primary: Color32::from_rgb(0, 100, 149),        // #006495
            on_primary: Color32::from_rgb(255, 255, 255),   // #ffffff
            primary_container: Color32::from_rgb(203, 230, 255), // #cbe6ff
            on_primary_container: Color32::from_rgb(0, 30, 48),  // #001e30
            secondary_container: Color32::from_rgb(217, 229, 242),// #d9e5f2
            on_secondary_container: Color32::from_rgb(17, 29, 40),// #111d28
            tertiary: Color32::from_rgb(112, 67, 142),      // #70438e
            tertiary_container: Color32::from_rgb(244, 217, 255), // #f4d9ff
            text: Color32::from_rgb(22, 28, 34),            // #161c22
            text_muted: Color32::from_rgb(88, 101, 115),    // #586573
            outline_color: Color32::from_rgba_unmultiplied(0, 0, 0, 24),
        }
    }

    pub fn classic() -> Self {
        Self {
            kind: ThemeKind::Classic,
            is_dark: true,
            bg: Color32::from_rgb(9, 13, 22),               // #090D16
            surface: Color32::from_rgb(15, 23, 42),         // #0F172A
            surface_low: Color32::from_rgb(17, 28, 46),     // #111C2E
            surface_high: Color32::from_rgb(26, 42, 68),    // #1A2A44
            surface_highest: Color32::from_rgb(36, 56, 92), // #24385C
            primary: Color32::from_rgb(59, 130, 246),       // #3B82F6
            on_primary: Color32::from_rgb(255, 255, 255),
            primary_container: Color32::from_rgb(29, 78, 216), // #1D4ED8
            on_primary_container: Color32::from_rgb(219, 234, 254),
            secondary_container: Color32::from_rgb(30, 41, 59),
            on_secondary_container: Color32::from_rgb(241, 245, 249),
            tertiary: Color32::from_rgb(168, 85, 247),
            tertiary_container: Color32::from_rgb(88, 28, 135),
            text: Color32::from_rgb(248, 250, 252),         // #F8FAFC
            text_muted: Color32::from_rgb(100, 116, 139),   // #64748B
            outline_color: Color32::from_rgb(28, 43, 68),    // #1C2B44
        }
    }

    pub fn outline(&self) -> Stroke {
        Stroke::new(1.0, self.outline_color)
    }

    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut style = (*ctx.global_style()).clone();
        if self.is_dark {
            style.visuals = egui::Visuals::dark();
        } else {
            style.visuals = egui::Visuals::light();
        }
        style.visuals.override_text_color = Some(self.text);
        style.visuals.window_fill = self.bg;
        style.visuals.panel_fill = self.bg;
        style.visuals.widgets.inactive.bg_fill = self.surface_low;
        style.visuals.widgets.inactive.weak_bg_fill = self.surface_low;
        style.visuals.widgets.hovered.bg_fill = self.surface_high;
        style.visuals.widgets.hovered.weak_bg_fill = self.surface_high;
        style.visuals.widgets.active.bg_fill = self.surface_highest;
        style.visuals.widgets.active.weak_bg_fill = self.surface_highest;
        style.visuals.selection.stroke = Stroke::new(1.0, self.primary);
        ctx.set_global_style(style);
    }
}
