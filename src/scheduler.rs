#![cfg(target_os = "android")]

use jni::{
    objects::{GlobalRef, JClass, JObject, JString, JValue},
    JavaVM,
};
use std::sync::OnceLock;
use thiserror::Error;

static JAVA_VM: OnceLock<JavaVM> = OnceLock::new();
/// Cached global reference to MainActivity, stored in JNI_OnLoad while the app's
/// class loader is active. Required because FindClass on background threads only
/// has access to the bootstrap class loader, which doesn't include app classes.
static MAIN_ACTIVITY_CLASS: OnceLock<GlobalRef> = OnceLock::new();

/// Called by Android when the native library is first loaded (on the main thread).
#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    let vm = match JavaVM::from_raw(vm) {
        Ok(vm) => vm,
        Err(_) => return jni::sys::JNI_ERR,
    };

    // get_env() (not attach) — JNI_OnLoad already runs on an attached thread.
    if let Ok(mut env) = vm.get_env() {
        match env.find_class("dev/dioxus/main/MainActivity") {
            Ok(class) => match env.new_global_ref(class) {
                Ok(global) => {
                    MAIN_ACTIVITY_CLASS.set(global).ok();
                }
                Err(e) => log::error!("JNI_OnLoad: new_global_ref failed: {e}"),
            },
            Err(e) => log::error!("JNI_OnLoad: find_class failed: {e}"),
        }
    }

    JAVA_VM.set(vm).ok();
    jni::sys::JNI_VERSION_1_6
}

/// Returns the app-private files directory by calling MainActivity.getFilesDir().
/// Falls back to /data/local/tmp if anything goes wrong, logging the specific failure.
pub fn android_db_path() -> std::path::PathBuf {
    let fallback = std::path::PathBuf::from("/data/local/tmp/alarms.db");

    let vm = match JAVA_VM.get() {
        Some(v) => v,
        None => {
            log::error!("android_db_path: JavaVM not initialized");
            return fallback;
        }
    };
    let class = match MAIN_ACTIVITY_CLASS.get() {
        Some(c) => c,
        None => {
            log::error!("android_db_path: MainActivity class not cached");
            return fallback;
        }
    };
    let mut env = match vm.attach_current_thread() {
        Ok(e) => e,
        Err(e) => {
            log::error!("android_db_path: attach_current_thread failed: {e}");
            return fallback;
        }
    };

    let jclass = unsafe { JClass::from_raw(class.as_raw()) };
    let result = match env.call_static_method(jclass, "getFilesDir", "()Ljava/lang/String;", &[]) {
        Ok(r) => r,
        Err(e) => {
            log::error!("android_db_path: getFilesDir call failed: {e}");
            if env.exception_check().unwrap_or(false) {
                env.exception_describe().ok();
                env.exception_clear().ok();
            }
            return fallback;
        }
    };

    let obj = match result.l() {
        Ok(o) => o,
        Err(e) => {
            log::error!("android_db_path: result not an object: {e}");
            return fallback;
        }
    };
    let jstr = JString::from(obj);
    let path: String = match env.get_string(&jstr) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("android_db_path: string conversion failed: {e}");
            return fallback;
        }
    };

    let full = std::path::PathBuf::from(path).join("alarms.db");
    log::info!("android_db_path: {:?}", full);
    full
}

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("JavaVM not available — JNI_OnLoad has not run yet")]
    NoVm,
    #[error("jni error: {0}")]
    Jni(#[from] jni::errors::Error),
}

pub fn schedule_alarm(
    alarm_id: i64,
    time: &str,
    confirm_ms: i64,
    refire_ms: i64,
) -> Result<(), SchedulerError> {
    let vm = JAVA_VM.get().ok_or(SchedulerError::NoVm)?;
    let class = MAIN_ACTIVITY_CLASS.get().ok_or(SchedulerError::NoVm)?;
    let mut env = vm.attach_current_thread()?;
    let jclass = unsafe { JClass::from_raw(class.as_raw()) };

    let jtime = env.new_string(time)?;
    env.call_static_method(
        jclass,
        "scheduleAlarmFromHHMM",
        "(Ljava/lang/String;IJJ)V",
        &[
            JValue::Object(&JObject::from(jtime)),
            JValue::Int(alarm_id as i32),
            JValue::Long(confirm_ms),
            JValue::Long(refire_ms),
        ],
    )?;

    Ok(())
}

pub fn get_global_settings() -> Result<(i64, i64), SchedulerError> {
    let vm = JAVA_VM.get().ok_or(SchedulerError::NoVm)?;
    let class = MAIN_ACTIVITY_CLASS.get().ok_or(SchedulerError::NoVm)?;
    let mut env = vm.attach_current_thread()?;
    let jclass = unsafe { JClass::from_raw(class.as_raw()) };

    let confirm = env
        .call_static_method(&jclass, "getGlobalConfirmDelayMs", "()J", &[])?
        .j()?;
    let refire = env
        .call_static_method(&jclass, "getGlobalRefireDelayMs", "()J", &[])?
        .j()?;
    Ok((confirm, refire))
}

pub fn set_global_settings(confirm_ms: i64, refire_ms: i64) -> Result<(), SchedulerError> {
    let vm = JAVA_VM.get().ok_or(SchedulerError::NoVm)?;
    let class = MAIN_ACTIVITY_CLASS.get().ok_or(SchedulerError::NoVm)?;
    let mut env = vm.attach_current_thread()?;
    let jclass = unsafe { JClass::from_raw(class.as_raw()) };

    env.call_static_method(
        jclass,
        "setGlobalSettings",
        "(JJ)V",
        &[JValue::Long(confirm_ms), JValue::Long(refire_ms)],
    )?;
    Ok(())
}

pub fn cancel_alarm(alarm_id: i64) -> Result<(), SchedulerError> {
    let vm = JAVA_VM.get().ok_or(SchedulerError::NoVm)?;
    let class = MAIN_ACTIVITY_CLASS.get().ok_or(SchedulerError::NoVm)?;
    let mut env = vm.attach_current_thread()?;
    let jclass = unsafe { JClass::from_raw(class.as_raw()) };

    env.call_static_method(
        jclass,
        "cancelAlarm",
        "(I)V",
        &[JValue::Int(alarm_id as i32)],
    )?;

    Ok(())
}
