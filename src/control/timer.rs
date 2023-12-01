use utoipa::ToSchema;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use std::{time::Duration, sync::Arc};
use crate::control::state;
use crate::util::{govee_api::SetState, timeday::TimeDay, fn_queue};

pub type SimpleTimers = Arc<Mutex<Vec<SimpleTimer>>>;
pub type Timers = Arc<Mutex<Vec<Timer>>>;

pub struct SimpleTimer {
    timeday: TimeDay,
    description: &'static str,
    /// take govee_queue as argument
    function: fn_queue::Element
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
pub struct Timer {
    enable: bool,
    #[schema(inline)]
    timeday: TimeDay,
    #[schema(inline)]
    action: TimerAction
}
impl Timer {
    pub fn get_timeday(&self) -> &TimeDay { &self.timeday }
    pub fn get_action(&self) -> &TimerAction { &self.action }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
// results in { "type": "Sunrise", "params": { "duration_min": ... }}
#[serde(tag = "type", content = "params")]
pub enum TimerAction {
    /// alarm for waking up with sunrise.
    /// sunrise finishes on `timeday` and then stays on for `stay_on_for_min` before turning off.
    /// brightness will return to default before turning off.
    /// nightlamp will be on for `nightlamp_min`,
    /// with `sleep_min` between the nightlamp turning off and the sunrise finishing.
    Sunrise {
        /// how long the sunrise should be
        #[schema(minimum = 1, maximum = 32767)] // i16::MAX
        duration_min: u16,
        /// how long the finished sunrise should stay on
        #[schema(minimum = 0, maximum = 32767)] // i16::MAX
        stay_on_for_min: u16,
        /// time between nightlamp turning off and sunrise finishing.
        /// has to be `>= duration_min`.
        #[schema(maximum = 32767)] // i16::MAX
        sleep_min: u16,
        /// how long the nightlamp should stay on
        #[schema(minimum = 1, maximum = 32767)] // i16::MAX
        nightlamp_min: u16
    },
    /// set brightness to default for night and color to nice warm white.
    Nightlamp,
    /// set power state to given value
    PowerState { power: bool },
    /// set color state to given value
    ColorState {
        #[schema(minimum = 0, maximum = 255)]
        r: u8,
        #[schema(minimum = 0, maximum = 255)]
        g: u8,
        #[schema(minimum = 0, maximum = 255)]
        b: u8
    },
    // TODO action for setting brightness
}

/// convert `Timer`s to `SimpleTimer`s and save them to `simple_timers`.
pub async fn process_timers(timers: &Timers, simple_timers: &SimpleTimers) {
    let mut generated_timers: Vec<SimpleTimer> = vec![];

    let timers = timers.lock().await;
    for timer in timers.iter() {
        // skip disabled timers
        if !timer.enable { continue; }
        match timer.action {
            TimerAction::Sunrise { duration_min, stay_on_for_min, sleep_min, nightlamp_min } => {
                generated_timers.push(SimpleTimer {
                    description: "nightlamp on",
                    timeday: timer.timeday.shift_time(
                        0,
                        - (sleep_min as i16) - (nightlamp_min as i16)
                    ),
                    function: Arc::new(move |govee_queue|
                        state::nightlamp(govee_queue))
                });
                generated_timers.push(SimpleTimer {
                    description: "nightlamp off",
                    timeday: timer.timeday.shift_time(
                        0,
                        - (sleep_min as i16)
                    ),
                    function: Arc::new(|govee_queue|
                        govee_queue.push_back(SetState::Power(false)))
                });
                generated_timers.push(SimpleTimer {
                    description: "sunrise",
                    timeday: timer.timeday.shift_time(
                        0,
                        - (duration_min as i16)
                    ),
                    function: Arc::new(move |govee_queue| {
                        state::sunrise(
                            govee_queue,
                            Duration::from_secs((duration_min as u64) * 60)
                        );
                    })
                });
                // turn off later (and set default brightness before)
                generated_timers.push(SimpleTimer {
                    description: "turn off",
                    timeday: timer.timeday.shift_time(
                        0,
                        stay_on_for_min as i16
                    ),
                    function: Arc::new(|govee_queue| {
                        use crate::constants::brightness::DAY;
                        govee_queue.push_back(SetState::Brightness(DAY));
                        govee_queue.push_back(SetState::Power(false));
                    })
                });
            },
            TimerAction::Nightlamp => {
                generated_timers.push(SimpleTimer {
                    description: "nightlamp on",
                    timeday: timer.timeday.clone(),
                    function: Arc::new(move |govee_queue|
                        state::nightlamp(govee_queue))
                });
            },
            TimerAction::PowerState { power } => {
                generated_timers.push(SimpleTimer {
                    description: "set power",
                    timeday: timer.timeday.clone(),
                    function: Arc::new(move |govee_queue|
                        govee_queue.push_back(SetState::Power(power)))
                });
            },
            TimerAction::ColorState { r, g, b } => {
                generated_timers.push(SimpleTimer {
                    description: "set color",
                    timeday: timer.timeday.clone(),
                    function: Arc::new(move |govee_queue|
                        govee_queue.push_back(SetState::Color((r, g, b))))
                });
            },
        }
    }

    println!("updated timers with {} generated simple timer(s) from {} complex timer(s)", generated_timers.len(), timers.len());
    if generated_timers.len() > 0 {
        for timer in generated_timers.iter() {
            println!("{}: {}", timer.timeday, timer.description);
        }
    }

    *simple_timers.lock().await = generated_timers;
}

/// if a timer matches the current date/time: push its function to the function queue.
/// update `last_checked` with the current time if timers have been checked.
pub async fn check_timers(simple_timers: &SimpleTimers, mut function_queue: &fn_queue::Queue, last_checked: &mut TimeDay) {
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
            println!("matched timer for {}, calling function...", timer.timeday);
        }
    }

    *last_checked = now;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test; // async tests

    #[test]
    async fn sunrise_timer_processing() {
        let simple_timers: SimpleTimers = Arc::new(Mutex::new(vec![]));
        let timers: Timers = Arc::new(Mutex::new(vec![Timer {
            enable: true,
            timeday: TimeDay::new(7, 0, vec![0]),
            action: TimerAction::Sunrise {
                duration_min: 20,
                stay_on_for_min: 5,
                sleep_min: (60 * 8) + 30,
                nightlamp_min: 60
            }
        }]));
        process_timers(&timers, &simple_timers).await;
        let simple_timers = simple_timers.lock().await;
        assert_eq!(simple_timers.len(), 4);
        assert!(simple_timers.iter().any(|t| t.timeday == TimeDay::new(21, 30, vec![6])));
        assert!(simple_timers.iter().any(|t| t.timeday == TimeDay::new(22, 30, vec![6])));
        assert!(simple_timers.iter().any(|t| t.timeday == TimeDay::new( 6, 40, vec![0])));
        assert!(simple_timers.iter().any(|t| t.timeday == TimeDay::new( 7,  5, vec![0])));
    }
}