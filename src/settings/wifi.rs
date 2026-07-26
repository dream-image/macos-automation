use objc2_core_wlan::CWWiFiClient;

pub fn disable() -> Result<(), String> {
    let client = unsafe { CWWiFiClient::sharedWiFiClient() };
    let interface = unsafe { client.interface() }.ok_or("当前设备没有可用的 Wi-Fi 接口")?;

    unsafe { interface.setPower_error(false) }
        .map_err(|error| error.localizedDescription().to_string())
}
