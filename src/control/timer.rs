use crate::control::{fn_queue, timeday::TimeDay};
use std::sync::{Arc, Mutex};

pub type SimpleTimers = Arc<Mutex<Vec<SimpleTimer>>>;
pub type Timers = Arc<Mutex<Vec<Timer>>>;

pub struct SimpleTimer {
    // TODO implement struct
    timeday: TimeDay,
    /// take function_queue as argument
    function: Box<dyn Fn(&mut fn_queue::Queue) -> () + Send>
}

#[derive(Debug, Clone, Copy)]
pub enum Timer {
    /// alarm for waking up with sunrise
    Alarm {
        // TODO implement enum
    }
}

/// get a copy of the current value of `timers`
pub fn get_copy(timers: &Timers) -> Vec<Timer> {
    timers.lock().unwrap().clone()
}

/// convert `Timer`s to `SimpleTimer`s and save them to `simple_timers`.
pub fn process_timers(timers: &Timers, simple_timers: &mut SimpleTimers) {
    let mut simple_timers = simple_timers.lock().unwrap();
    simple_timers.clear();
    // TODO implement function
}

/// if a timer matches the current date/time: push its function to the function queue.
pub fn check_timers(simple_timers: &SimpleTimers, function_queue: &mut fn_queue::Queue) {
    // TODO implement function
}