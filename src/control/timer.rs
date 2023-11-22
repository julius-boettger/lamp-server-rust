use crate::control::{fn_queue, timeday::TimeDay};
use std::sync::{Arc, Mutex};

pub type SimpleTimers = Arc<Mutex<Vec<SimpleTimer>>>;
pub type Timers = Arc<Mutex<Vec<Timer>>>;

pub struct SimpleTimer {
    timeday: TimeDay,
    /// take function_queue as argument
    function: Box<dyn Fn(&mut fn_queue::Queue) -> () + Send>
}

#[derive(Debug, Clone)]
pub struct Timer {
    enable: bool,
    timeday: TimeDay,
    action: TimerAction
}

#[derive(Debug, Clone)]
enum TimerAction {
    /// alarm for waking up with sunrise.
    /// sunrise finishes on `timeday` and then stays on for `stay_on_for_min`
    /// before returning to default brightness and turning off.
    Sunrise {
        /// how long the sunrise should be
        duration_min: u8,
        /// how long the finished sunrise should stay on
        stay_on_for_min: u8
    }
}

/// get a clone of the current value of `timers`
pub fn get_clone(timers: &Timers) -> Vec<Timer> {
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