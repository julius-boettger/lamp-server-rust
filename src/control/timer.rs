use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use crate::control::{
    self,
    fn_queue,
    SetState,
    timeday::TimeDay
};

pub type SimpleTimers = Arc<Mutex<Vec<SimpleTimer>>>;
pub type Timers = Arc<Mutex<Vec<Timer>>>;

pub struct SimpleTimer {
    pub timeday: TimeDay,
    /// take govee_queue as argument
    pub function: fn_queue::Element
}

#[derive(Debug, Clone)]
pub struct Timer {
    pub enable: bool,
    pub timeday: TimeDay,
    pub action: TimerAction
}

#[derive(Debug, Clone)]
pub enum TimerAction {
    /// alarm for waking up with sunrise.
    /// sunrise finishes on `timeday` and then stays on for `stay_on_for_min` before turning off.
    Sunrise {
        /// how long the sunrise should be
        duration_min: u8,
        /// how long the finished sunrise should stay on
        stay_on_for_min: u8
    }
}

/// get a clone of the current value of `timers`
pub async fn get_clone(timers: &Timers) -> Vec<Timer> {
    timers.lock().await.clone()
}

/// convert `Timer`s to `SimpleTimer`s and save them to `simple_timers`.
pub async fn process_timers(timers: &Timers, simple_timers: &mut SimpleTimers) {
    let mut generated_timers: Vec<SimpleTimer> = vec![];

    for timer in timers.lock().await.iter() {
        // skip disabled timers
        if !timer.enable { continue; }
        match timer.action {
            TimerAction::Sunrise { duration_min, stay_on_for_min } => {
                // actual sunrise
                generated_timers.push(SimpleTimer {
                    timeday: timer.timeday.shift_time(
                        0,
                        - (duration_min as i8)
                    ),
                    function: Arc::new(move |govee_queue| {
                        control::generate_sunrise(
                            govee_queue,
                            Duration::from_secs((duration_min as u64) * 60)
                        );
                    })
                });
                // turn off later
                generated_timers.push(SimpleTimer {
                    timeday: timer.timeday.shift_time(
                        0,
                        stay_on_for_min as i8
                    ),
                    function: Arc::new(|govee_queue|
                        govee_queue.push_back(SetState::Power(false)))
                });
            }
        }
    }

    *simple_timers.lock().await = generated_timers;
}

/// if a timer matches the current date/time: push its function to the function queue.
/// update `last_checked` with the current time if timers have been checked.
pub async fn check_timers(simple_timers: &SimpleTimers, mut function_queue: &mut fn_queue::Queue, last_checked: &mut TimeDay) {
    let now = TimeDay::now();
    // if timers have already been checked this minute
    if now == *last_checked {
        return;
    }

    for timer in simple_timers.lock().await.iter() {
        if timer.timeday.get_days().contains(&now.get_days()[0])
        && timer.timeday.get_hour() == now.get_hour()
        && timer.timeday.get_minute() == now.get_minute() {
            fn_queue::enqueue(&mut function_queue, Arc::clone(&timer.function)).await;
            println!("matched timer for {:02}:{:02} on days {:?}",
                timer.timeday.get_hour(),
                timer.timeday.get_minute(),
                timer.timeday.get_days()
            );
        }
    }

    *last_checked = now;
}