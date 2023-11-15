use crate::util;
use crate::res::secrets::govee;

type RGBColor = (u8, u8, u8);

enum SetState {
    Color(RGBColor),
    Brightness(u8),
    Power(bool)
}

fn set_state(status: SetState) {
    if cfg!(govee_debug) {
        // TODO handle debug mode
        return;
    }
}

async fn get_power_state() {
    let url = format!("https://developer-api.govee.com/v1/devices/state?device={}&model={}", govee::DEVICE, govee::MODEL);
    let response = util::send_api_request(url.as_str(), util::HttpMethod::Get).await;
    println!("{:?}", response.unwrap().text().await);
    // TODO error handling, parse json response, return result
}