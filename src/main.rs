#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn main() -> eframe::Result {
    env_logger::init();
    tvremote::run_desktop()
}

// Android uses `android_main` in the library; WebAssembly uses `tvremote-web`.
#[cfg(any(target_os = "android", target_arch = "wasm32"))]
fn main() {}

