use std::ffi::c_char;

use objc2_core_foundation::{
    CFBoolean, CFPreferencesCopyAppValue, CFPreferencesSetValue, CFPreferencesSynchronize,
    CFString, kCFPreferencesCurrentHost, kCFPreferencesCurrentUser,
};

const DOMAIN: &str = "com.apple.coreservices.useractivityd";
const ADVERTISING_KEY: &str = "ActivityAdvertisingAllowed";
const RECEIVING_KEY: &str = "ActivityReceivingAllowed";
const CHANGED_NOTIFICATION: &std::ffi::CStr =
    c"LSUserActivityManagerActivityContinuationIsEnabledChangedNotification";

unsafe extern "C" {
    fn notify_post(name: *const c_char) -> u32;
}

pub fn is_enabled() -> bool {
    let domain = CFString::from_static_str(DOMAIN);
    let advertising_key = CFString::from_static_str(ADVERTISING_KEY);
    let receiving_key = CFString::from_static_str(RECEIVING_KEY);

    CFPreferencesSynchronize(&domain, unsafe { kCFPreferencesCurrentUser }, unsafe {
        kCFPreferencesCurrentHost
    });

    let advertising = preference(&advertising_key, &domain);
    let receiving = preference(&receiving_key, &domain);

    match (advertising, receiving) {
        (Some(advertising), Some(receiving)) => advertising && receiving,
        _ => true,
    }
}

pub fn set_enabled(enabled: bool) -> Result<(), &'static str> {
    let domain = CFString::from_static_str(DOMAIN);
    let advertising_key = CFString::from_static_str(ADVERTISING_KEY);
    let receiving_key = CFString::from_static_str(RECEIVING_KEY);
    let value = CFBoolean::new(enabled);
    let user = unsafe { kCFPreferencesCurrentUser };
    let host = unsafe { kCFPreferencesCurrentHost };

    unsafe {
        CFPreferencesSetValue(&advertising_key, Some(value), &domain, user, host);
        CFPreferencesSetValue(&receiving_key, Some(value), &domain, user, host);
    }

    if !CFPreferencesSynchronize(&domain, user, host) {
        return Err("无法保存系统偏好设置");
    }

    if unsafe { notify_post(CHANGED_NOTIFICATION.as_ptr()) } != 0 {
        return Err("无法通知接力服务");
    }

    Ok(())
}

fn preference(key: &CFString, domain: &CFString) -> Option<bool> {
    CFPreferencesCopyAppValue(key, domain)?
        .downcast::<CFBoolean>()
        .ok()
        .map(|value| value.as_bool())
}
