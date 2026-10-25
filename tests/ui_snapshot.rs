use egui_kittest::Harness;
use tvremote::app::RemoteApp;
use tvremote::ui::state::{ActiveSheet, ActiveTab};
use tvremote::ui::UiMode;

fn snap(name: &str, w: f32, h: f32, mode: UiMode) {
    snap_with_setup(name, w, h, mode, |_| {});
}

fn snap_with_setup<F: FnOnce(&mut RemoteApp) + 'static>(name: &str, w: f32, h: f32, mode: UiMode, setup: F) {
    let tmp = std::env::temp_dir().join(format!("tvremote_snap_{}", name));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let mut harness = Harness::builder()
        .with_size(eframe::egui::vec2(w, h))
        .build_eframe(move |cc| {
            let mut app = RemoteApp::new(cc, tmp);
            app.ui_mgr.mode = mode;
            setup(&mut app);
            app
        });

    harness.run();
    harness.run_steps(4);
    harness.snapshot(name);
}

// ---------------- Minimal Apple TV Edition Tests ----------------

#[test]
fn snap_minimal_360x640() {
    snap("minimal_360x640", 360.0, 640.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_360x760() {
    snap("minimal_360x760", 360.0, 760.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_360x800() {
    snap("minimal_360x800", 360.0, 800.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_384x854() {
    snap("minimal_384x854", 384.0, 854.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_393x873() {
    snap("minimal_393x873", 393.0, 873.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_412x915() {
    snap("minimal_412x915", 412.0, 915.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_430x932() {
    snap("minimal_430x932", 430.0, 932.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_tablet_768x1024() {
    snap("minimal_tablet_768x1024", 768.0, 1024.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_desktop_1280x800() {
    snap("minimal_desktop_1280x800", 1280.0, 800.0, UiMode::Minimal);
}

#[test]
fn snap_minimal_sheet_apps_360x640() {
    snap_with_setup("minimal_sheet_apps_360x640", 360.0, 640.0, UiMode::Minimal, |app| {
        app.ui_mgr.state.active_sheet = ActiveSheet::Apps;
    });
}

// ---------------- Material 3 Expressive UI Tests ----------------

#[test]
fn snap_expressive_360x640() {
    snap("expressive_360x640", 360.0, 640.0, UiMode::Expressive);
}

#[test]
fn snap_expressive_360x800() {
    snap("expressive_360x800", 360.0, 800.0, UiMode::Expressive);
}

#[test]
fn snap_expressive_393x873() {
    snap("expressive_393x873", 393.0, 873.0, UiMode::Expressive);
}

#[test]
fn snap_expressive_412x915() {
    snap("expressive_412x915", 412.0, 915.0, UiMode::Expressive);
}

#[test]
fn snap_expressive_430x932() {
    snap("expressive_430x932", 430.0, 932.0, UiMode::Expressive);
}

#[test]
fn snap_expressive_tablet_768x1024() {
    snap("expressive_tablet_768x1024", 768.0, 1024.0, UiMode::Expressive);
}

#[test]
fn snap_expressive_desktop_1280x800() {
    snap("expressive_desktop_1280x800", 1280.0, 800.0, UiMode::Expressive);
}

#[test]
fn snap_expressive_apps_360x640() {
    snap_with_setup("expressive_apps_360x640", 360.0, 640.0, UiMode::Expressive, |app| {
        app.ui_mgr.state.tab = ActiveTab::Apps;
    });
}

#[test]
fn snap_expressive_keypad_360x640() {
    snap_with_setup("expressive_keypad_360x640", 360.0, 640.0, UiMode::Expressive, |app| {
        app.ui_mgr.state.tab = ActiveTab::Keypad;
    });
}

#[test]
fn snap_expressive_sheet_options_360x640() {
    snap_with_setup("expressive_sheet_options_360x640", 360.0, 640.0, UiMode::Expressive, |app| {
        app.ui_mgr.state.active_sheet = ActiveSheet::Options;
    });
}

// ---------------- Advanced Full-Screen Mode Tests ----------------

#[test]
fn snap_advanced_360x640() {
    snap("advanced_360x640", 360.0, 640.0, UiMode::Advanced);
}

#[test]
fn snap_advanced_412x915() {
    snap("advanced_412x915", 412.0, 915.0, UiMode::Advanced);
}

#[test]
fn snap_advanced_430x932() {
    snap("advanced_430x932", 430.0, 932.0, UiMode::Advanced);
}

#[test]
fn snap_advanced_desktop_1280x800() {
    snap("advanced_desktop_1280x800", 1280.0, 800.0, UiMode::Advanced);
}
