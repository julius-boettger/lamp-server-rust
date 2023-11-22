use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::control::govee::SetState;

/// take govee_queue as argument
pub type Element = &'static (dyn Fn(&mut VecDeque<SetState>) -> () + Send + Sync);
// TODO use tokio::sync::Mutex?
pub type Queue = Arc<Mutex<VecDeque<Element>>>;

/// call and then remove each function, starting from the front.
pub fn call_all(function_queue: &mut Queue, govee_queue: &mut VecDeque<SetState>) {
    let mut function_queue = function_queue.lock().unwrap();
    // call all functions
    while !function_queue.is_empty() {
        function_queue.pop_front().unwrap()(govee_queue);
    }
}

pub fn enqueue(function_queue: &mut Queue, function: Element) {
    let mut function_queue = function_queue.lock().unwrap();
    function_queue.push_back(function);
}