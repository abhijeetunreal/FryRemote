//! TV Remote: a Google TV / Android TV remote control written in Rust.

pub mod app;
pub mod controller;
#[cfg(not(target_arch = "wasm32"))]
pub mod creds;
#[cfg(not(target_arch = "wasm32"))]
pub mod discovery;
pub mod haptics;
pub mod icons;
pub mod keys;
#[cfg(not(target_arch = "wasm32"))]
pub mod pairing;
#[cfg(not(target_arch = "wasm32"))]
pub mod proto;
#[cfg(not(target_arch = "wasm32"))]
pub mod remote;
pub mod state;
pub mod store;
#[cfg(not(target_arch = "wasm32"))]
pub mod tls;
#[cfg(not(target_arch = "wasm32"))]
pub mod wire;


#[cfg(target_os = "android")]
mod android;

/// Entry point used by Android's NativeActivity.
#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info).with_tag("fryremote"),
    );
    let data_dir = app
        .internal_data_path()
        .unwrap_or_else(|| std::path::PathBuf::from("/data/local/tmp"));
    android::acquire_multicast_lock();

    let options = eframe::NativeOptions { android_app: Some(app), ..Default::default() };
    if let Err(e) = eframe::run_native(
        "FryRemote",
        options,
        Box::new(move |cc| Ok(Box::new(app::RemoteApp::new(cc, data_dir)))),
    ) {
        log::error!("eframe exited with error: {e}");
    }
}

/// Desktop entry point (handy for trying the UI on a computer).
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn run_desktop() -> eframe::Result {
    let dir = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(".tvremote");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("FryRemote")
            .with_inner_size([390.0, 820.0])
            .with_icon(app_icon())
            .with_resizable(true)
            .with_active(true)
            .with_visible(true),
        ..Default::default()
    };
    eframe::run_native(
        "FryRemote",
        options,
        Box::new(move |cc| Ok(Box::new(app::RemoteApp::new(cc, dir)))),
    )
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn app_icon() -> eframe::egui::IconData {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - 15.5) / 14.5;
            let dy = (y as f32 - 15.5) / 14.5;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= 1.0 {
                if y >= 14 && y <= 27 && x >= 9 && x <= 22 {
                    if (x >= 14 && x <= 17 && y >= 19 && y <= 25) || (x >= 11 && x <= 20 && y >= 21 && y <= 23) {
                        rgba.extend_from_slice(&[18, 28, 45, 255]);
                    } else if y <= 17 && x >= 14 && x <= 17 {
                        rgba.extend_from_slice(&[41, 121, 255, 255]);
                    } else {
                        rgba.extend_from_slice(&[240, 246, 255, 255]);
                    }
                } else if y >= 4 && y <= 13 && x >= 6 && x <= 25 {
                    let wave_d = ((x as f32 - 15.5).powi(2) + (y as f32 - 14.0).powi(2)).sqrt();
                    if (wave_d >= 8.5 && wave_d <= 11.0) || (wave_d >= 5.0 && wave_d <= 7.0) || (wave_d >= 1.5 && wave_d <= 3.2) {
                        rgba.extend_from_slice(&[41, 121, 255, 255]);
                    } else {
                        rgba.extend_from_slice(&[9, 14, 24, 255]);
                    }
                } else {
                    rgba.extend_from_slice(&[9, 14, 24, 255]);
                }
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    eframe::egui::IconData { rgba, width, height }
}
