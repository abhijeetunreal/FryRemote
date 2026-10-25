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
fn try_android_feedback(style: HapticStyle, duration_ms: i64) -> Result<(), Box<dyn std::error::Error>> {
    use jni::objects::{JObject, JValue};
    use jni::JavaVM;

    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast())? };
    let mut env = vm.attach_current_thread()?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    let window = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?.l()?;
    if !window.is_null() {
        let decor = env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?.l()?;
        if !decor.is_null() {
            // 1. Play native Android click sound if enabled (SoundEffectConstants.CLICK = 0)
            if is_sound_enabled() {
                let _ = env.call_method(&decor, "playSoundEffect", "(I)V", &[JValue::Int(0)]);
            }

            // 2. Perform native view haptic feedback if haptics enabled
            if is_enabled() {
                // HapticFeedbackConstants:
                // LONG_PRESS = 0, VIRTUAL_KEY = 1, KEYBOARD_TAP = 3, CLOCK_TICK = 4
                let feedback_constant = match style {
                    HapticStyle::Tick => 4,
                    HapticStyle::Light => 3,
                    HapticStyle::Medium => 1,
                    HapticStyle::Heavy => 0,
                };

                // FLAG_IGNORE_VIEW_SETTING = 1 to guarantee execution
                let performed = env
                    .call_method(
                        &decor,
                        "performHapticFeedback",
                        "(II)Z",
                        &[JValue::Int(feedback_constant), JValue::Int(1)],
                    )
                    .and_then(|v| v.z())
                    .unwrap_or(false);

                if performed {
                    return Ok(());
                }

                // Fallback to single argument performHapticFeedback(int)
                let performed_single = env
                    .call_method(&decor, "performHapticFeedback", "(I)Z", &[JValue::Int(feedback_constant)])
                    .and_then(|v| v.z())
                    .unwrap_or(false);

                if performed_single {
                    return Ok(());
                }
            }
        }
    }

    // 3. Fallback to direct Vibrator service
    if is_enabled() {
        let service_name = env.new_string("vibrator")?;
        let vibrator = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )?
            .l()?;

        if !vibrator.is_null() {
            env.call_method(&vibrator, "vibrate", "(J)V", &[JValue::Long(duration_ms)])?;
        }
    }

    Ok(())
}
