use objc2_core_wlan::CWWiFiClient;

pub fn disable() -> Result<(), String> {
    set_power(false)
}

pub fn enable() -> Result<(), String> {
    set_power(true)
}

fn set_power(enabled: bool) -> Result<(), String> {
    let client = unsafe { CWWiFiClient::sharedWiFiClient() };
    let interface = unsafe { client.interface() }.ok_or("当前设备没有可用的 Wi-Fi 接口")?;

    unsafe { interface.setPower_error(enabled) }
        .map_err(|error| error.localizedDescription().to_string())
}
