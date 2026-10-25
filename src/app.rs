//! FryRemote Application Root
//!
//! Orchestrates the TV Controller and dispatches rendering to the modular
//! UI Manager (supporting Material 3 Expressive and Classic Advanced Mode).

use crate::controller::{Controller, Notify};
use crate::keys;
use crate::state::*;
use crate::ui::common::widgets::keypad;
use crate::ui::{ThemePalette, UiManager};
use eframe::egui::{self, Color32, RichText, Ui};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::Arc;

// Backward-compatible Theme Colors
pub const BG: Color32 = Color32::from_rgb(9, 13, 22);
pub const CARD_BG: Color32 = Color32::from_rgb(15, 23, 42);
pub const BTN_BG: Color32 = Color32::from_rgb(17, 28, 46);
pub const BTN_HOVER: Color32 = Color32::from_rgb(26, 42, 68);
pub const BTN_ACTIVE: Color32 = Color32::from_rgb(36, 56, 92);
pub const BORDER: Color32 = Color32::from_rgb(28, 43, 68);
pub const ACCENT: Color32 = Color32::from_rgb(59, 130, 246);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(96, 165, 250);
pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(29, 78, 216);
pub const GREEN_TEXT: Color32 = Color32::from_rgb(34, 197, 94);
pub const GREEN_BG: Color32 = Color32::from_rgb(11, 39, 32);
pub const GREEN_BORDER: Color32 = Color32::from_rgb(18, 71, 56);
pub const RED_POWER: Color32 = Color32::from_rgb(239, 68, 68);
pub const TEXT_WHITE: Color32 = Color32::from_rgb(248, 250, 252);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
pub const ICON_SLATE: Color32 = Color32::from_rgb(148, 163, 184);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Remote,
    Apps,
    Touchpad,
    More,
}

#[derive(Clone, Debug)]
pub enum Screen {
    Remote,
    Pair(Tv),
}

pub struct RemoteApp {
    pub ctrl: Controller,
    pub screen: Screen,
    pub ui_mgr: UiManager,
    auto_scanned: bool,
    code_input: String,
}

impl RemoteApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(cc: &eframe::CreationContext<'_>, data_dir: PathBuf) -> Self {
        let ctx = cc.egui_ctx.clone();
        let notify: Notify = Arc::new(move || ctx.request_repaint());
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app = Self {
            ctrl: Controller::new(data_dir, notify),
            screen: Screen::Remote,
            ui_mgr: UiManager::new(),
            auto_scanned: false,
            code_input: String::new(),
        };

        // If we have saved TVs, auto-connect to the first paired TV
        if let Some(tv) = app.ctrl.store.tvs.iter().find(|t| t.paired).cloned() {
            app.ctrl.connect(&tv);
        }

        app
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_web(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        let notify: Notify = Arc::new(move || ctx.request_repaint());
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut app = Self {
            ctrl: Controller::new_web(notify),
            screen: Screen::Remote,
            ui_mgr: UiManager::new(),
            auto_scanned: false,
            code_input: String::new(),
        };

        if let Some(tv) = app.ctrl.store.tvs.first().cloned() {
            app.ctrl.connect(&tv);
        }

        app
    }
}

impl eframe::App for RemoteApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ctrl.poll();
        let snap = self.ctrl.snapshot();

        if !self.auto_scanned && self.ctrl.creds_ready() {
            self.auto_scanned = true;
            self.ctrl.scan();
        }

        // Physical keyboard event listener: captures key presses and sends to Android TV
        let has_text_focus = ui.memory(|m| m.focused().is_some());
        if !has_text_focus && !matches!(self.screen, Screen::Pair(_)) {
            let events = ui.input(|i| i.events.clone());
            for event in events {
                match event {
                    egui::Event::Text(t) => {
                        if !t.is_empty() {
                            self.ctrl.send_text(&t);
                        }
                    }
                    egui::Event::Key { key, pressed: true, .. } => {
                        match key {
                            egui::Key::Backspace => {
                                self.ctrl.key(keys::DEL);
                            }
                            egui::Key::Enter => {
                                self.ctrl.key(keys::ENTER);
                            }
                            egui::Key::Space => {
                                self.ctrl.send_text(" ");
                            }
                            egui::Key::ArrowUp => {
                                self.ctrl.key(keys::DPAD_UP);
                            }
                            egui::Key::ArrowDown => {
                                self.ctrl.key(keys::DPAD_DOWN);
                            }
                            egui::Key::ArrowLeft => {
                                self.ctrl.key(keys::DPAD_LEFT);
                            }
                            egui::Key::ArrowRight => {
                                self.ctrl.key(keys::DPAD_RIGHT);
                            }
                            egui::Key::Escape => {
                                self.ctrl.key(keys::BACK);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(self.ui_mgr.theme.dot_color()).inner_margin(0.0))
            .show(ui, |ui| {
                match self.screen.clone() {
                    Screen::Pair(tv) => {
                        let mut next: Option<Screen> = None;
                        self.pair_screen(ui, &snap, &tv, &mut next);
                        if let Some(n) = next {
                            self.screen = n;
                        }
                    }
                    Screen::Remote => {
                        self.ui_mgr.ui(ui, &mut self.ctrl, &snap, &mut self.screen);
                    }
                }
            });
    }
}

impl RemoteApp {
    fn pair_screen(&mut self, ui: &mut Ui, snap: &Shared, tv: &Tv, next: &mut Option<Screen>) {
        let palette = ThemePalette::for_kind(self.ui_mgr.theme);
        ui.painter().rect_filled(ui.max_rect(), 0.0, palette.bg);

        let w = ui.available_width().min(400.0);
        let start_x = (ui.available_width() - w) * 0.5;

        ui.horizontal(|ui| {
            ui.add_space(start_x.max(0.0));
            ui.vertical(|ui| {
                ui.set_width(w);
                ui.add_space(20.0);
                ui.heading(RichText::new(format!("Pair with {}", tv.name)).color(palette.text));
                ui.label(RichText::new("A 6-character code is showing on your TV screen.").color(palette.text_muted).size(13.0));
                ui.add_space(14.0);

                match &snap.pairing {
                    PairingUi::None | PairingUi::Starting => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(RichText::new("Connecting to the TV...").color(palette.text));
                        });
                    }
                    PairingUi::WaitingForCode | PairingUi::WrongCode | PairingUi::Verifying => {
                        if snap.pairing == PairingUi::WrongCode {
                            ui.colored_label(Color32::from_rgb(255, 95, 95), "That code doesn't match. Check the TV and try again.");
                        }
                        let shown = format!("{:_<6}", self.code_input.to_uppercase());
                        ui.label(RichText::new(shown).size(38.0).monospace().strong().color(palette.primary));
                        keypad(ui, &mut self.code_input, true, 6);

                        let verifying = snap.pairing == PairingUi::Verifying;
                        if verifying {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(RichText::new("Verifying code...").color(palette.text));
                            });
                        }
                        let ready = self.code_input.len() == 6 && !verifying;
                        let submit_btn = egui::Button::new(RichText::new("Submit Code").size(16.0).color(palette.on_primary))
                            .fill(palette.primary)
                            .corner_radius(16.0)
                            .min_size(eframe::egui::vec2(w, 46.0));

                        if ui.add_enabled(ready, submit_btn).clicked() {
                            self.ctrl.submit_code(&self.code_input);
                        }
                    }
                    PairingUi::Done => {
                        ui.colored_label(GREEN_TEXT, "Paired successfully!");
                        let mut tv = tv.clone();
                        tv.paired = true;
                        self.ctrl.store.mark_paired(&tv.host);
                        self.ctrl.connect(&tv);
                        *next = Some(Screen::Remote);
                    }
                    PairingUi::Error(e) => {
                        ui.colored_label(Color32::from_rgb(255, 95, 95), format!("Pairing failed: {e}"));
                        if ui.button("Try again").clicked() {
                            self.code_input.clear();
                            self.ctrl.start_pairing(tv);
                        }
                    }
                }

                ui.add_space(16.0);
                if ui.button(RichText::new("Cancel").color(palette.text_muted)).clicked() {
                    self.ctrl.cancel_pairing();
                    *next = Some(Screen::Remote);
                }
            });
        });
    }
}
