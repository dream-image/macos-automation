use std::cell::OnceCell;
use std::ffi::c_int;
use std::thread;
use std::time::Duration;

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_core_bluetooth::CBCentralManager;

thread_local! {
    static PERMISSION_MANAGER: OnceCell<Retained<CBCentralManager>> = const { OnceCell::new() };
}

#[link(name = "IOBluetooth", kind = "framework")]
unsafe extern "C" {
    fn IOBluetoothPreferencesAvailable() -> c_int;
    fn IOBluetoothPreferenceGetControllerPowerState() -> c_int;
    fn IOBluetoothPreferenceSetControllerPowerState(state: c_int);
}

pub fn request_permission() {
    PERMISSION_MANAGER.with(|manager| {
        manager.get_or_init(|| unsafe { CBCentralManager::init(CBCentralManager::alloc()) });
    });
}

pub fn disable() -> Result<(), String> {
    set_power(false)
}

pub fn enable() -> Result<(), String> {
    set_power(true)
}

fn set_power(enabled: bool) -> Result<(), String> {
    let target_state = if enabled { 1 } else { 0 };

    unsafe {
        if IOBluetoothPreferencesAvailable() == 0 {
            return Err("当前设备没有可用的蓝牙控制器".into());
        }

        if IOBluetoothPreferenceGetControllerPowerState() == target_state {
            return Ok(());
        }

        IOBluetoothPreferenceSetControllerPowerState(target_state);
    }

    for _ in 0..20 {
        thread::sleep(Duration::from_millis(100));
        if unsafe { IOBluetoothPreferenceGetControllerPowerState() } == target_state {
            return Ok(());
        }
    }

    let action = if enabled { "打开" } else { "关闭" };
    Err(format!("蓝牙状态未在超时时间内{action}"))
}
