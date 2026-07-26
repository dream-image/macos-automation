use objc2_foundation::{NSString, NSUserDefaults, ns_string};

use crate::settings::{bluetooth, wifi};

#[derive(Clone, Copy)]
pub enum Setting {
    BluetoothOnShutdown,
    WifiOnShutdown,
}

impl Setting {
    fn key(self) -> &'static NSString {
        match self {
            Self::BluetoothOnShutdown => ns_string!("bluetooth_on_shutdown"),
            Self::WifiOnShutdown => ns_string!("wifi_on_shutdown"),
        }
    }
}

pub fn is_enabled(setting: Setting) -> bool {
    NSUserDefaults::standardUserDefaults().boolForKey(setting.key())
}

pub fn set_enabled(setting: Setting, enabled: bool) {
    NSUserDefaults::standardUserDefaults().setBool_forKey(enabled, setting.key());
}

pub fn handle_shutdown() {
    if is_enabled(Setting::BluetoothOnShutdown)
        && let Err(error) = bluetooth::disable()
    {
        eprintln!("关闭蓝牙失败: {error}");
    }

    if is_enabled(Setting::WifiOnShutdown)
        && let Err(error) = wifi::disable()
    {
        eprintln!("关闭 Wi-Fi 失败: {error}");
    }
}
