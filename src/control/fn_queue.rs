use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;
use crate::control::govee::SetState;

/// take govee_queue as argument
pub type Element = Arc<dyn Fn(&mut VecDeque<SetState>) -> () + Send + Sync>;
// TODO use tokio::sync::Mutex?
pub type Queue = Arc<Mutex<VecDeque<Element>>>;

/// call and then remove each function, starting from the front.
pub async fn call_all(function_queue: &mut Queue, govee_queue: &mut VecDeque<SetState>) {
    let mut function_queue = function_queue.lock().await;
    // call all functions
    while !function_queue.is_empty() {
        function_queue.pop_front().unwrap()(govee_queue);
    }
}

pub async fn enqueue(function_queue: &mut Queue, function: Element) {
    function_queue.lock().await.push_back(function);
}