use std::{time::Duration, collections::VecDeque};
use crate::constants;
use crate::util::govee_api::SetState;

/// set brightness to default for night and color to nice warm white
pub fn nightlamp(govee_queue: &mut VecDeque<SetState>) {
    use constants::{brightness::NIGHT, colors::NIGHTLAMP};
    println!("activating nightlamp...");
    govee_queue.push_back(SetState::Brightness(NIGHT));
    govee_queue.push_back(SetState::Color(NIGHTLAMP));
}

/// set brightness to default for day and color to pleasant orange
pub fn daylamp(govee_queue: &mut VecDeque<SetState>) {
    use constants::{brightness::DAY, colors::DAYLAMP};
    println!("activating daylamp...");
    govee_queue.push_back(SetState::Brightness(DAY));
    govee_queue.push_back(SetState::Color(DAYLAMP));
}

/// append states to show a bright orange with high brightness for about 20 seconds and then turn off again
pub fn reminder(govee_queue: &mut VecDeque<SetState>) {
    use constants::{colors::REMINDER as COLOR, brightness::REMINDER as BRIGHTNESS};
    println!("activating reminder...");
    govee_queue.push_back(SetState::Color(COLOR));
    govee_queue.push_back(SetState::Brightness(BRIGHTNESS));
    govee_queue.push_back(SetState::Power(true)); // do nothing
    govee_queue.push_back(SetState::Power(false));
}

/// append states for a sunrise of given duration
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn sunrise(govee_queue: &mut VecDeque<SetState>, sunrise_duration: Duration) {
    use constants::govee::{API_REQUEST_INTERVAL, AVG_SET_STATE_DURATION};
    use constants::sunrise::{govee_brightness, hsv_color};

    // number of `SetState`s to generate for brightness and color each.
    // f64 type is needed for later calculations.
    let state_amount = (
        sunrise_duration.as_millis() /
        (API_REQUEST_INTERVAL.as_millis() + AVG_SET_STATE_DURATION.as_millis()) /
        2 // for brightness and color each
    ) as f64;

    let brightness_step = (govee_brightness::STOP - govee_brightness::START) / (state_amount - 1.0);
    let saturation_step = (hsv_color::saturation::START - hsv_color::saturation::STOP) / (state_amount - 1.0);

    for i in 0 .. state_amount as u32 {
        let iteration = f64::from(i);
        govee_queue.push_back(SetState::Brightness(
            brightness_step.mul_add(iteration, govee_brightness::START)
                .round() as u8
        ));
        govee_queue.push_back(SetState::Color(
            hsv::hsv_to_rgb(
                hsv_color::HUE,
                saturation_step.mul_add(-iteration, hsv_color::saturation::START),
                hsv_color::VALUE,
            )
        ));
    }

    println!("generated {} sunrise states for {:.1} min sunrise",
        state_amount as u32,
        sunrise_duration.as_secs_f32() / 60f32
    );
}