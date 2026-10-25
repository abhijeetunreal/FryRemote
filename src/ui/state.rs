//! UI State Management for FryRemote: holds view state across tabs, sheets, and modes.

use eframe::egui::{Pos2, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveTab {
    Remote,
    Apps,
    Keypad,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdvancedTab {
    Remote,
    Apps,
    Touchpad,
    More,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PadMode {
    Dpad,
    Swipe,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveSheet {
    None,
    Options,
    Device,
    Keyboard,
    MediaScrubber,
    Voice,
    UxNotes,
}

#[derive(Clone, Debug)]
pub struct UiState {
    pub tab: ActiveTab,
    pub advanced_tab: AdvancedTab,
    pub pad_mode: PadMode,
    pub active_sheet: ActiveSheet,

    pub tv_powered: bool,
    pub is_playing: bool,
    pub is_muted: bool,
    pub volume_level: f32, // 0.0 to 1.0
    pub current_channel: u32,
    pub numpad_buffer: String,

    pub feedback_text: Option<String>,
    pub feedback_until: f64,

    pub active_app_name: String,
    pub active_media_title: String,
    pub active_app_subtitle: String,

    pub touch_acc: Vec2,
    pub touch_ripples: Vec<(Pos2, f64)>,

    pub keyboard_input: String,
    pub typed_text: String,
    pub code_input: String,
    pub ip_input: String,
    pub show_ip_pad: bool,
    pub show_tv_selector: bool,
    pub media_scrubber_pos: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            tab: ActiveTab::Remote,
            advanced_tab: AdvancedTab::Remote,
            pad_mode: PadMode::Dpad,
            active_sheet: ActiveSheet::None,

            tv_powered: true,
            is_playing: true,
            is_muted: false,
            volume_level: 0.45,
            current_channel: 104,
            numpad_buffer: "104".to_string(),

            feedback_text: None,
            feedback_until: 0.0,

            active_app_name: "Netflix".to_string(),
            active_media_title: "Stranger Things — S4:E9".to_string(),
            active_app_subtitle: "4K HDR".to_string(),

            touch_acc: Vec2::ZERO,
            touch_ripples: Vec::new(),

            keyboard_input: String::new(),
            typed_text: String::new(),
            code_input: String::new(),
            ip_input: String::new(),
            show_ip_pad: false,
            show_tv_selector: false,
            media_scrubber_pos: 38.0,
        }
    }
}

impl UiState {
    pub fn send_command_feedback(&mut self, text: impl Into<String>, now: f64) {
        self.feedback_text = Some(text.into());
        self.feedback_until = now + 1.4;
    }

    pub fn current_feedback<'a>(&'a self, now: f64, default_status: &'a str) -> &'a str {
        if now < self.feedback_until {
            if let Some(fb) = &self.feedback_text {
                return fb.as_str();
            }
        }
        default_status
    }

    pub fn toggle_sheet(&mut self, sheet: ActiveSheet) {
        if self.active_sheet == sheet {
            self.active_sheet = ActiveSheet::None;
        } else {
            self.active_sheet = sheet;
        }
    }

    pub fn close_sheet(&mut self) {
        self.active_sheet = ActiveSheet::None;
    }
}
