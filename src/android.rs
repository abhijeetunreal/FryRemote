//! Android-specific glue.

use jni::objects::{JObject, JValue};
use jni::JavaVM;

/// Wi-Fi drivers drop multicast packets unless a MulticastLock is held, which
/// would make mDNS discovery find nothing. The lock lives for the whole process.
pub fn acquire_multicast_lock() {
    if let Err(e) = try_acquire() {
        log::warn!("could not acquire multicast lock: {e}");
    }
}

fn try_acquire() -> Result<(), Box<dyn std::error::Error>> {
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

    let res = (|| -> Result<(), Box<dyn std::error::Error>> {
        let service_name = env.new_string("wifi")?;
        let wifi = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )?
            .l()?;
        if wifi.is_null() {
            return Ok(());
        }
        let tag = env.new_string("fryremote")?;
        let lock = env
            .call_method(
                &wifi,
                "createMulticastLock",
                "(Ljava/lang/String;)Landroid/net/wifi/WifiManager$MulticastLock;",
                &[JValue::Object(&tag)],
            )?
            .l()?;
        if !lock.is_null() {
            env.call_method(&lock, "setReferenceCounted", "(Z)V", &[JValue::Bool(0)])?;
            env.call_method(&lock, "acquire", "()V", &[])?;
            // Keep the lock object alive for the life of the process.
            std::mem::forget(env.new_global_ref(lock)?);
        }
        Ok(())
    })();

    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
    res
}
