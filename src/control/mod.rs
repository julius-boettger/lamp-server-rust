pub mod web;
pub mod timer;
pub mod fn_queue;

use crate::util::govee;
use crate::res::constants;
use std::sync::Arc;
use govee::SetState;
use tokio::sync::Mutex;
use std::time::Duration;
use std::collections::VecDeque;

/// never terminates
pub async fn main_loop() {

    use constants::govee::API_REQUEST_INTERVAL;
    use constants::sunrise::*;
    use std::thread::sleep;
    use timer::SimpleTimers;
    use crate::util::timeday::TimeDay;

    if govee_brightness::START >= govee_brightness::STOP {
        panic!("sunrise brightness has to start smaller than it stops");
    }
    if hsv_color::saturation::START <= hsv_color::saturation::STOP {
        panic!("sunrise color saturation has to start larger than it stops");
    }

    if cfg!(govee_debug) {
        println!("GOVEE_DEBUG is enabled: not sending PUT requests to Govee API");
    }

    // queue of `SetState`s of which the first one will be executed each iteration
    let mut govee_queue: VecDeque<SetState> = VecDeque::new();

    // queue of functions to be executed once at the start of the next loop.
    // all functions will be executed and then removed from the queue, starting from the front.
    // each function has access to govee_queue.
    // confusing type is for thread safety.
    let mut function_queue: fn_queue::Queue = Arc::new(Mutex::new(VecDeque::new()));

    // collection of timers to be checked every minute.
    // if a timer matches the current time its function will be pushed to the function queue.
    let simple_timers: SimpleTimers = Arc::new(Mutex::new(vec![]));

    // will be updated by timer::check_timers() to avoid matching timers more than once per minute
    let mut last_checked_time = TimeDay::now().shift_time(0, -1);

    // "fire and forget" web server start
    tokio::spawn(web::start_server(
        Arc::clone(&function_queue),
        Arc::clone(&simple_timers)
    ));

    // wait before starting loop to avoid reaching rate limits when restarting frequently
    sleep(API_REQUEST_INTERVAL);

    // actual main loop
    loop {
        timer::check_timers(&simple_timers, &mut function_queue, &mut last_checked_time).await;

        fn_queue::call_all(&mut function_queue, &mut govee_queue).await;

        if !govee_queue.is_empty() {
            let success = govee::set_state(*govee_queue.front().unwrap()).await;
            if success {
                govee_queue.pop_front();
            } else {
                println!("setting state failed, trying again");
            }
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

    println!("generated {} sunrise states for {:.1} min sunrise",
        state_amount as u32,
        sunrise_duration.as_secs_f32() / 60f32
    );
}