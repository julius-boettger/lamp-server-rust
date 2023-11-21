use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::control::govee::SetState;

// TODO convert to struct with methods

pub type FunctionQueueElement = Box<dyn Fn(&mut VecDeque<SetState>) -> () + Send>;
pub type FunctionQueue = Arc<Mutex<VecDeque<FunctionQueueElement>>>;

/// call and then remove each function, starting from the front.
pub fn call_all(function_queue: &mut FunctionQueue, govee_queue: &mut VecDeque<SetState>) {
    let mut function_queue = function_queue.lock().unwrap();
    // call all functions
    while !function_queue.is_empty() {
        function_queue.pop_front().unwrap()(govee_queue);
    }
}

pub fn enqueue(function_queue: &mut FunctionQueue, function: FunctionQueueElement) {
    let mut function_queue = function_queue.lock().unwrap();
    function_queue.push_back(function);
}