use std::ffi::c_int;
use std::thread;
use std::time::Duration;

#[link(name = "IOBluetooth", kind = "framework")]
unsafe extern "C" {
    fn IOBluetoothPreferencesAvailable() -> c_int;
    fn IOBluetoothPreferenceGetControllerPowerState() -> c_int;
    fn IOBluetoothPreferenceSetControllerPowerState(state: c_int);
}

pub fn disable() -> Result<(), String> {
    unsafe {
        if IOBluetoothPreferencesAvailable() == 0 {
            return Err("当前设备没有可用的蓝牙控制器".into());
        }

        if IOBluetoothPreferenceGetControllerPowerState() == 0 {
            return Ok(());
        }

        IOBluetoothPreferenceSetControllerPowerState(0);
    }

    for _ in 0..20 {
        thread::sleep(Duration::from_millis(100));
        if unsafe { IOBluetoothPreferenceGetControllerPowerState() } == 0 {
            return Ok(());
        }
    }

    Err("蓝牙状态未在超时时间内关闭".into())
}
