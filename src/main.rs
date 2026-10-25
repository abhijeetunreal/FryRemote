#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result {
    env_logger::init();
    tvremote::run_desktop()
}

// The Android build uses the library's `android_main`; this binary is desktop-only.
#[cfg(target_os = "android")]
fn main() {}
