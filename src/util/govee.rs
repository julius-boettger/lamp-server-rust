use crate::util;
use crate::res::secrets::govee;
use crate::view::out::govee_debug;

pub type RGBColor = (u8, u8, u8);
/// from 0 to 100
pub type Brightness = u8;
pub type Power = bool;

#[derive(Debug)]
pub enum SetState {
    Color(RGBColor),
    Brightness(Brightness),
    Power(Power)
}

#[derive(Debug)]
pub struct GetState {
    color: RGBColor,
    brightness: Brightness,
    power: Power
}

impl Default for GetState {
    fn default() -> Self {
        Self {
            color: (0, 0, 0),
            brightness: 100,
            power: false
        }
    }
}

// TODO implement function
pub async fn set_state(state: SetState) {
    if cfg!(govee_debug) {
        govee_debug::println(format!("setting state: {:?}", state));
        return;
    }

    // TODO construct json body from state and stringify

    //util::send_api_request(
    //    util::HttpMethod::Put,
    //    url.as_str(),
    //    Some(vec![("Govee-API-Key", govee::API_KEY), ("Content-Type", "application/json")])
    //).await;
}

pub async fn get_state() -> GetState {
    if cfg!(govee_debug) {
        govee_debug::println(format!("using default GetState: {:?}", GetState::default()));
        return GetState::default();
    }

    let url = format!("https://developer-api.govee.com/v1/devices/state?device={}&model={}", govee::DEVICE, govee::MODEL);
    let result = util::send_api_request(
        util::HttpMethod::Get,
        url.as_str(),
        Some(vec![("Govee-API-Key", govee::API_KEY)])
    ).await;

    let Ok(json) = result else {
        return GetState::default();
    };

    // TODO get parameters from json result
    println!("{:?}", json);

    GetState {
        // very dependent on govee api!
        color: (0, 0, 0),
        brightness: 100,
        power: false
    }
}