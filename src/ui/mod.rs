//! Modular UI Architecture for FryRemote.
//!
//! Provides extensible UI skinning where different visual styles (such as
//! Material 3 Expressive and Classic Advanced Mode) can coexist and be switched
//! at runtime without altering core networking, protocol, or controller logic.
//!
//! ### Adding a New UI Skin in the Future:
//! 1. Create a submodule directory under `src/ui/<your_skin_name>/`
//! 2. Implement the render function with signature:
//!    `render(ui: &mut Ui, palette: &ThemePalette, state: &mut UiState, ctrl: &mut Controller, snap: &Shared, ...)`
//! 3. Add a new variant to `UiMode` enum below.
//! 4. Add the dispatch branch in `UiManager::ui`.

pub mod advanced;
pub mod common;
pub mod expressive;
pub mod state;
pub mod theme;

pub use state::UiState;
pub use theme::{ThemeKind, ThemePalette};

use crate::controller::Controller;
use crate::state::Shared;
use eframe::egui::Ui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    /// Primary UI: Material 3 Expressive redesign matching Google TV aesthetics.
    Expressive,
    /// Advanced Mode: Full-screen responsive layout with all legacy controls.
    Advanced,
}

impl Default for UiMode {
    fn default() -> Self {
        UiMode::Expressive
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
            mode: UiMode::Expressive,
            theme: ThemeKind::Sky,
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
            UiMode::Expressive => ThemePalette::for_kind(self.theme),
            UiMode::Advanced => ThemePalette::classic(),
        };

        if self.last_applied_theme != Some(self.theme) || self.last_applied_mode != Some(self.mode) {
            palette.apply_to_ctx(ui.ctx());
            self.last_applied_theme = Some(self.theme);
            self.last_applied_mode = Some(self.mode);
        }

        // Render according to active UI mode
        match self.mode {
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
