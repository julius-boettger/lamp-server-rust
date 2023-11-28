pub mod web;
pub mod state;
pub mod timer;
pub mod fn_queue;

/// never terminates
pub async fn main_loop() {
    use std::thread::sleep;
    use crate::res::constants::{sunrise::*, govee::API_REQUEST_INTERVAL};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use timer::SimpleTimers;
    use std::collections::VecDeque;
    use crate::util::timeday::TimeDay;
    use crate::util::govee::{self, SetState};

    if govee_brightness::START >= govee_brightness::STOP {
        panic!("sunrise brightness has to start smaller than it stops");
    }
    if hsv_color::saturation::START <= hsv_color::saturation::STOP {
        panic!("sunrise color saturation has to start larger than it stops");
    }

    if cfg!(govee_debug) {
        println!("GOVEE_DEBUG is enabled: not sending PUT requests to Govee API");
    }

    // queue of `SetState`s of which the first one will be used for a Govee API call each iteration
    let mut govee_queue: VecDeque<SetState> = VecDeque::new();

    // queue of functions to be called once at the start of the next loop.
    // all functions will be called and then removed from the queue, starting from the front.
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