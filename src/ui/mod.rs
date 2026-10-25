//! Modular UI Architecture for FryRemote.
//!
//! Provides extensible UI skinning where different visual styles (Apple TV Minimal,
//! Material 3 Expressive, and Classic Advanced Mode) coexist and are switched
//! at runtime in a seamless loop without altering core networking, protocol, or controller logic.

pub mod advanced;
pub mod common;
pub mod expressive;
pub mod minimal;
pub mod state;
pub mod theme;

pub use state::UiState;
pub use theme::{ThemeKind, ThemePalette};

use crate::controller::Controller;
use crate::state::Shared;
use eframe::egui::Ui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    /// Primary Default UI: Minimal Apple TV Edition
    Minimal,
    /// Material 3 Expressive UI matching Google TV aesthetics
    Expressive,
    /// Advanced Mode: Full-screen responsive layout with all legacy controls
    Advanced,
}

impl UiMode {
    /// Cycles to the next UI layout in a continuous loop:
    /// Minimal (Apple TV) -> Expressive (Material 3) -> Advanced -> Minimal...
    pub fn next(&self) -> Self {
        match self {
            UiMode::Minimal => UiMode::Expressive,
            UiMode::Expressive => UiMode::Advanced,
            UiMode::Advanced => UiMode::Minimal,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            UiMode::Minimal => "Apple TV Minimal",
            UiMode::Expressive => "Material 3 Expressive",
            UiMode::Advanced => "Advanced Full-Screen",
        }
    }
}

impl Default for UiMode {
    fn default() -> Self {
        UiMode::Minimal
    }
}

pub struct UiManager {
    pub mode: UiMode,
    pub theme: ThemeKind,
    pub state: UiState,
    last_applied_theme: Option<ThemeKind>,
    last_applied_mode: Option<UiMode>,
}

impl UiManager {
    pub fn new() -> Self {
        Self {
            mode: UiMode::Minimal,
            theme: ThemeKind::Obsidian,
            state: UiState::default(),
            last_applied_theme: None,
            last_applied_mode: None,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        ctrl: &mut Controller,
        snap: &Shared,
        screen: &mut crate::app::Screen,
    ) {
        // Apply theme styling when theme or mode changes
        let palette = match self.mode {
            UiMode::Minimal => ThemePalette::for_kind(self.theme),
            UiMode::Expressive => {
                // If switching into Expressive from an Apple TV theme, ensure an Expressive theme
                let expr_theme = match self.theme {
                    ThemeKind::Obsidian | ThemeKind::Silver | ThemeKind::Titanium => ThemeKind::Sky,
                    other => other,
                };
                ThemePalette::for_kind(expr_theme)
            }
            UiMode::Advanced => ThemePalette::classic(),
        };

        if self.last_applied_theme != Some(self.theme) || self.last_applied_mode != Some(self.mode) {
            palette.apply_to_ctx(ui.ctx());
            self.last_applied_theme = Some(self.theme);
            self.last_applied_mode = Some(self.mode);
        }

        // Render according to active UI mode
        match self.mode {
            UiMode::Minimal => {
                minimal::render_minimal_ui(
                    ui,
                    &palette,
                    &mut self.state,
                    ctrl,
                    snap,
                    &mut self.theme,
                    &mut self.mode,
                    screen,
                );
            }
            UiMode::Expressive => {
                expressive::render_expressive_ui(
                    ui,
                    &palette,
                    &mut self.state,
                    ctrl,
                    snap,
                    &mut self.theme,
                    &mut self.mode,
                    screen,
                );
            }
            UiMode::Advanced => {
                advanced::render_advanced_ui(
                    ui,
                    &mut self.state,
                    ctrl,
                    snap,
                    &mut self.mode,
                    screen,
                );
            }
        }
    }
}
