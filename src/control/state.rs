use crate::res::constants;
use std::time::Duration;
use std::collections::VecDeque;
use crate::util::govee::SetState;

/// set brightness to default for night and color to nice warm white
pub fn nightlamp(govee_queue: &mut VecDeque<SetState>) {
    use constants::{govee::default_brightness::NIGHT, colors::NIGHTLAMP};
    govee_queue.push_back(SetState::Brightness(NIGHT));
    govee_queue.push_back(SetState::Color(NIGHTLAMP));
}

// TODO configure nightlamp for sunrise
/// append `SetState`s for a sunrise of given duration to `govee_queue`
pub fn sunrise(govee_queue: &mut VecDeque<SetState>, sunrise_duration: Duration) {
    use constants::{govee::*, sunrise::*};

    // number of `SetState`s to generate for brightness and color each.
    // f64 type is needed for later calculations.
    let state_amount = (
        sunrise_duration.as_millis() /
        (API_REQUEST_INTERVAL.as_millis() + AVG_SET_STATE_DURATION.as_millis()) /
        2 // for brightness and color each
    ) as f64;

    let brightness_step = (govee_brightness::STOP - govee_brightness::START) / (state_amount - 1.0);
    let saturation_step = (hsv_color::saturation::START - hsv_color::saturation::STOP) / (state_amount - 1.0);

    // `while` because `for` is not possible with f64
    let mut iteration = 0.0f64;
    while iteration < state_amount {
        govee_queue.push_back(SetState::Brightness(
            (govee_brightness::START + (brightness_step * iteration))
                .round() as u8
        ));
        govee_queue.push_back(SetState::Color(
            hsv::hsv_to_rgb(
                hsv_color::HUE,
                hsv_color::saturation::START - (saturation_step * iteration),
                hsv_color::VALUE,
            )
        ));
        iteration += 1.0;
    }

    println!("generated {} sunrise states for {:.1} min sunrise",
        state_amount as u32,
        sunrise_duration.as_secs_f32() / 60f32
    );
}