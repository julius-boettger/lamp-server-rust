pub mod web;

use crate::util::govee;
use crate::res::constants;
use std::collections::VecDeque;

/// never terminates
pub async fn main_loop() {

    // queue of `SetState`s of which the first one will be executed each iteration
    let mut govee_queue: VecDeque<govee::SetState> = VecDeque::new();

    loop {
        if !govee_queue.is_empty() {
            govee::set_state(govee_queue.pop_front().unwrap()).await;
        }
        println!("----- waiting -----");
        std::thread::sleep(constants::govee::API_REQUEST_INTERVAL);
    }
}