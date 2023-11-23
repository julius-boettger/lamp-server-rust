use crate::util;
use crate::res::constants;
use crate::res::secrets::govee;

#[derive(Debug, Clone, Copy)]
pub enum SetState {
    Color((u8, u8, u8)),
    /// from 1 to 100
    Brightness(u8),
    Power(bool)
}

#[derive(
    Debug,
    serde::Serialize, // to axum::Json
    utoipa::ToSchema  // to display in swagger-ui
)]
pub struct GetState {
    #[schema(min_items = 3, max_items = 3)]
    pub color: (u8, u8, u8),
    /// from 1 to 100
    #[schema(minimum = 1, maximum = 100)]
    pub brightness: u8,
    pub power: bool
}

/// limits brightness from 1 to 100.
/// returns success.
/// dependent on govee api.
/// only prints state and waits a little instead of setting it if `cfg!(govee_debug)`.
pub async fn set_state(state: SetState) -> bool {

    println!("setting state to {:?}", state);

    if cfg!(govee_debug) {
        // emulate request by waiting a bit
        std::thread::sleep(constants::govee::AVG_SET_STATE_DURATION);
        return true;
    }

    let url = String::from("https://developer-api.govee.com/v1/devices/control");

    let cmd_name = match state {
        SetState::Color(_) => "color",
        SetState::Brightness(_) => "brightness",
        SetState::Power(_) => "turn"
    };

    let cmd_value: serde_json::Value = match state {
        SetState::Color(color) => serde_json::json!({
            "r": color.0,
            "g": color.1,
            "b": color.2,
        }),
        SetState::Brightness(brightness) => brightness.clamp(1, 100).into(),
        SetState::Power(power) => (if power { "on" } else { "off" }).into()
    };

    let body = serde_json::json!({
        "device": govee::DEVICE,
        "model": govee::MODEL,
        "cmd": {
            "name": cmd_name,
            "value": cmd_value
        }
    }).to_string();

    let result = util::send_api_request(
        util::HttpMethod::Put(body),
        url.as_str(),
        Some(vec![("Govee-API-Key", govee::API_KEY), ("Content-Type", "application/json")])
    ).await;

    let Ok(json) = result else {
        return false;
    };

    // true if status code is 200
    json["code"].as_u64().unwrap() == 200
}

/// dependent on govee api.
pub async fn get_state() -> Result<GetState, ()> {
    let url = format!("https://developer-api.govee.com/v1/devices/state?device={}&model={}", govee::DEVICE, govee::MODEL);
    let result = util::send_api_request(
        util::HttpMethod::Get,
        url.as_str(),
        Some(vec![("Govee-API-Key", govee::API_KEY)])
    ).await;

    let Ok(json) = result else {
        return Err(());
    };

    let data = &json["data"]["properties"];
    let state = GetState {
        color: (
            data[3]["color"]["r"].as_u64().unwrap().try_into().unwrap(),
            data[3]["color"]["g"].as_u64().unwrap().try_into().unwrap(),
            data[3]["color"]["b"].as_u64().unwrap().try_into().unwrap()
        ),
        brightness: data[2]["brightness"].as_u64().unwrap().try_into().unwrap(),
        power: data[1]["powerState"].as_str().unwrap() == "on"
    };

    println!("got state {:?}", state);
    Ok(state)
}