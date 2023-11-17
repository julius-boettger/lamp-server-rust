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

    let url = String::from("https://developer-api.govee.com/v1/devices/control");

    // TODO use cmd_name and cmd_value from state

    let body = serde_json::json!({
        "device": govee::DEVICE,
        "model": govee::MODEL,
        "cmd": {
            "name": "turn",
            "value": "on"
        }
    });

    let result = util::send_api_request(
        util::HttpMethod::Put(body.to_string()),
        url.as_str(),
        Some(vec![("Govee-API-Key", govee::API_KEY), ("Content-Type", "application/json")])
    ).await;

    println!("{:?}", result);
}

/// uses `GetState::default()` on error
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

    // very dependent on govee api!
    let data = &json["data"]["properties"];
    GetState {
        color: (
            data[3]["color"]["r"].as_u64().unwrap().try_into().unwrap(),
            data[3]["color"]["g"].as_u64().unwrap().try_into().unwrap(),
            data[3]["color"]["b"].as_u64().unwrap().try_into().unwrap()
        ),
        brightness: data[2]["brightness"].as_u64().unwrap().try_into().unwrap(),
        power: data[1]["powerState"].as_str().unwrap() == "on"
    }
}