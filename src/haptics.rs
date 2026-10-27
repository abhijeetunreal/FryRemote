//! Tactile haptic feedback and vibration for FryRemote buttons and gestures.

use std::sync::atomic::{AtomicBool, Ordering};

static HAPTICS_ENABLED: AtomicBool = AtomicBool::new(true);
static SOUND_ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HapticStyle {
    /// Light subtle click (e.g., standard keypress, keypad, nav bar switch)
    Light,
    /// Medium tactile click (e.g., volume steps, media buttons, function keys)
    Medium,
    /// Heavy impactful buzz (e.g., D-pad arrows, OK, Power toggle)
    Heavy,
    /// Quick micro-tick (e.g., slider drag step, touchpad gesture step)
    Tick,
}

/// Enable or disable haptic vibration feedback.
pub fn set_enabled(enabled: bool) {
    HAPTICS_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if haptic vibration feedback is enabled.
pub fn is_enabled() -> bool {
    HAPTICS_ENABLED.load(Ordering::Relaxed)
}

/// Enable or disable touch sound effects.
pub fn set_sound_enabled(enabled: bool) {
    SOUND_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if touch sound effects are enabled.
pub fn is_sound_enabled() -> bool {
    SOUND_ENABLED.load(Ordering::Relaxed)
}

/// Triggers tactile haptic vibration with the given style.
pub fn trigger(style: HapticStyle) {
    if !is_enabled() && !is_sound_enabled() {
        return;
    }

    let duration_ms = match style {
        HapticStyle::Tick => 12,
        HapticStyle::Light => 22,
        HapticStyle::Medium => 35,
        HapticStyle::Heavy => 55,
    };

    feedback(style, duration_ms);
}

#[cfg(target_os = "android")]
fn feedback(style: HapticStyle, duration_ms: i64) {
    if let Err(e) = try_android_feedback(style, duration_ms) {
        log::debug!("android haptics error: {e}");
    }
}

#[cfg(not(target_os = "android"))]
fn feedback(style: HapticStyle, duration_ms: i64) {
    // Non-android / desktop platform: trace log for testing
    log::trace!("haptic feedback triggered: {style:?} ({duration_ms}ms)");
}

#[cfg(target_os = "android")]
fn try_android_feedback(_style: HapticStyle, duration_ms: i64) -> Result<(), Box<dyn std::error::Error>> {
    use jni::objects::{JObject, JValue};
    use jni::JavaVM;

    let ctx = ndk_context::android_context();
    if ctx.vm().is_null() || ctx.context().is_null() {
        return Ok(());
    }

    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };
    if activity.is_null() {
        return Ok(());
    }

    // 1. Play native Android click sound if enabled via AudioManager (thread-safe, does not require UI thread)
    if is_sound_enabled() {
        let play_sound = || -> Result<(), Box<dyn std::error::Error>> {
            let audio_svc = env.new_string("audio")?;
            let audio_mgr = env
                .call_method(
                    &activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&audio_svc)],
                )?
                .l()?;

            if !audio_mgr.is_null() {
                // AudioManager.FX_KEY_CLICK = 0
                let _ = env.call_method(&audio_mgr, "playSoundEffect", "(I)V", &[JValue::Int(0)]);
            }
            Ok(())
        };

        let _ = play_sound();
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
    }

    // 2. Perform vibration feedback via Vibrator system service (thread-safe, does not touch View hierarchy)
    if is_enabled() {
        let vibrate = || -> Result<(), Box<dyn std::error::Error>> {
            let vib_svc = env.new_string("vibrator")?;
            let vibrator = env
                .call_method(
                    &activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&vib_svc)],
                )?
                .l()?;

            if !vibrator.is_null() {
                let has_vib = env
                    .call_method(&vibrator, "hasVibrator", "()Z", &[])
                    .and_then(|v| v.z())
                    .unwrap_or(true);

                if has_vib {
                    let _ = env.call_method(&vibrator, "vibrate", "(J)V", &[JValue::Long(duration_ms)]);
                }
            }
            Ok(())
        };

        let _ = vibrate();
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
    }

    Ok(())
}
