//! TV Remote: a Google TV / Android TV remote control written in Rust.

pub mod app;
pub mod controller;
pub mod creds;
pub mod discovery;
pub mod haptics;
pub mod icons;
pub mod keys;
pub mod pairing;
pub mod proto;
pub mod remote;
pub mod state;
pub mod store;
pub mod tls;
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
#[cfg(not(target_os = "android"))]
pub fn run_desktop() -> eframe::Result {
    let dir = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(".tvremote");
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([390.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "FryRemote",
        options,
        Box::new(move |cc| Ok(Box::new(app::RemoteApp::new(cc, dir)))),
    )
}
