use crate::util;
use crate::res::secrets::govee;

type RGBColor = (u8, u8, u8);

pub enum SetState {
    Color(RGBColor),
    Brightness(u8),
    Power(bool)
}

pub async fn set_state(status: SetState) {
    if cfg!(govee_debug) {
        // TODO handle debug mode
        return;
    }
    // TODO implement function
    //util::send_api_request(
    //    util::HttpMethod::Put,
    //    url.as_str(),
    //    Some(vec![("Govee-API-Key", govee::API_KEY), ("Content-Type", "application/json")])
    //).await;
}

pub async fn get_state() -> Result<serde_json::Value, &'static str> {
    if cfg!(govee_debug) {
        // TODO handle debug mode
        return Err("cannot run get_state() in debug mode");
    }

    let url = format!("https://developer-api.govee.com/v1/devices/state?device={}&model={}", govee::DEVICE, govee::MODEL);
    util::send_api_request(
        util::HttpMethod::Get,
        url.as_str(),
        Some(vec![("Govee-API-Key", govee::API_KEY)])
    ).await
}