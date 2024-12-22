pub mod web;
pub mod state;
pub mod timer;

/// one time setup
pub fn setup() {
    use crate::util::govee_secrets;
    use crate::constants::sunrise::*;

    // check sunrise constants
    if govee_brightness::START >= govee_brightness::STOP {
        panic!("sunrise brightness has to start smaller than it stops");
    }
    if hsv_color::saturation::START <= hsv_color::saturation::STOP {
        panic!("sunrise color saturation has to start larger than it stops");
    }

    // read govee secrets from config file
    govee_secrets::INSTANCE.set(govee_secrets::from_file()).unwrap();
    println!("SETUP: successfully loaded config from file");

    // check debug mode
    if cfg!(govee_debug) {
        println!("SETUP: GOVEE_DEBUG is enabled => not sending PUT requests to Govee API");
    }
}

/// never terminates
pub async fn main_loop() {
    use tokio::sync::Mutex;
    use timer::SimpleTimers;
    use std::{collections::VecDeque, sync::Arc, thread::sleep};
    use crate::constants::govee::API_REQUEST_INTERVAL;
    use crate::util::{fn_queue, timeday::TimeDay, govee_api::{self, SetState}};

    setup();

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

    // start webserver ("fire and forget" instead of "await")
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
            let success = govee_api::set_state(*govee_queue.front().unwrap()).await;
            match success {
                true  => { govee_queue.pop_front(); },
                false => println!("setting state failed, trying again")
            }
        }

        println!("----- waiting -----");
        sleep(API_REQUEST_INTERVAL);
    }
}