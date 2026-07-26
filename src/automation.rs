use std::thread;
use std::time::Duration;

use objc2_foundation::{NSString, NSUserDefaults, ns_string};

use crate::settings::{bluetooth, wifi};

#[derive(Clone, Copy)]
pub enum Setting {
    BluetoothOnSleep,
    WifiOnSleep,
}

impl Setting {
    fn key(self) -> &'static NSString {
        match self {
            Self::BluetoothOnSleep => ns_string!("bluetooth_on_sleep"),
            Self::WifiOnSleep => ns_string!("wifi_on_sleep"),
        }
    }
}

pub fn is_enabled(setting: Setting) -> bool {
    NSUserDefaults::standardUserDefaults().boolForKey(setting.key())
}

pub fn set_enabled(setting: Setting, enabled: bool) {
    NSUserDefaults::standardUserDefaults().setBool_forKey(enabled, setting.key());
}

pub fn request_permission(setting: Setting) {
    if let Setting::BluetoothOnSleep = setting {
        bluetooth::request_permission();
    }
}

pub fn handle_sleep() {
    if is_enabled(Setting::BluetoothOnSleep)
        && let Err(error) = bluetooth::disable()
    {
        eprintln!("关闭蓝牙失败: {error}");
    }

    if is_enabled(Setting::WifiOnSleep)
        && let Err(error) = wifi::disable()
    {
        eprintln!("关闭 Wi-Fi 失败: {error}");
    }
}

pub fn handle_wake() {
    let bluetooth_enabled = is_enabled(Setting::BluetoothOnSleep);
    let wifi_enabled = is_enabled(Setting::WifiOnSleep);

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));

        if bluetooth_enabled && let Err(error) = bluetooth::enable() {
            eprintln!("打开蓝牙失败: {error}");
        }

        if wifi_enabled && let Err(error) = wifi::enable() {
            eprintln!("打开 Wi-Fi 失败: {error}");
        }
    });
}
