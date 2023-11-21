pub mod web;
pub mod timer;
pub mod fn_queue;

use crate::util::govee;
use crate::res::constants;
use govee::SetState;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// never terminates
pub async fn main_loop() {
    {
        use constants::sunrise::*;
        if govee_brightness::START >= govee_brightness::STOP {
            panic!("sunrise brightness has to start smaller than it stops");
        }
        if hsv_color::saturation::START <= hsv_color::saturation::STOP {
            panic!("sunrise color saturation has to start larger than it stops");
        }
    };

    use crate::res::constants::govee::API_REQUEST_INTERVAL;
    use std::thread::sleep;

    // queue of `SetState`s of which the first one will be executed each iteration
    let mut govee_queue: VecDeque<SetState> = VecDeque::new();

    // queue of functions to be executed once at the start of the next loop.
    // all functions will be executed and then removed from the queue, starting from the front.
    // each function has access to govee_queue.
    // confusing type is for thread safety.
    let mut function_queue: fn_queue::Queue = Arc::new(Mutex::new(VecDeque::new()));

    // collection of timers to be checked every minute.
    // if a timer matches the current time its function will be pushed to the function queue.
    let simple_timers: timer::SimpleTimers = Arc::new(Mutex::new(Vec::new()));

    // "fire and forget" web server start
    tokio::spawn(web::start_server(
        Arc::clone(&function_queue),
        Arc::clone(&simple_timers)
    ));

    // wait before starting loop to avoid reaching rate limits when restarting frequently
    sleep(API_REQUEST_INTERVAL);

    // actual main loop
    loop {
        timer::check_timers(&simple_timers, &mut function_queue);

        fn_queue::call_all(&mut function_queue, &mut govee_queue);

        if !govee_queue.is_empty() {
            govee::set_state(govee_queue.pop_front().unwrap()).await;
        }

        println!("----- waiting -----");
        sleep(API_REQUEST_INTERVAL);
    }
}

/// append `SetState`s for a sunrise of given duration to `govee_queue`
fn generate_sunrise(govee_queue: &mut VecDeque<SetState>, sunrise_duration: Duration) {
    use crate::res::constants::govee::*;
    use crate::res::constants::sunrise::*;

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
}

// append `SetState::Power(true)`s to simulate doing nothing for the given duration
fn add_fillers(govee_queue: &mut VecDeque<SetState>, filler_duration: Duration) {
    use crate::res::constants::govee::*;

    let state_amount = filler_duration.as_millis() /
        (API_REQUEST_INTERVAL.as_millis() + AVG_SET_STATE_DURATION.as_millis());

    for _ in 0..state_amount {
        govee_queue.push_back(SetState::Power(true));
    }
}