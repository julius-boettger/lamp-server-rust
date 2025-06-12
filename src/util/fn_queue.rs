use tokio::sync::Mutex;
use crate::util::govee_api::SetState;
use std::{sync::Arc, collections::VecDeque};

/// take `govee_queue` as argument
pub type Element = Arc<dyn Fn(&mut VecDeque<SetState>) + Send + Sync>;
pub type Queue = Arc<Mutex<VecDeque<Element>>>;

/// call and then remove each function, starting from the front.
#[allow(clippy::significant_drop_tightening)]
pub async fn call_all(function_queue: &Queue, govee_queue: &mut VecDeque<SetState>) {
    let mut function_queue = function_queue.lock().await;
    // call all functions
    while !function_queue.is_empty() {
        function_queue.pop_front().unwrap()(govee_queue);
    }
}

pub async fn enqueue(function_queue: &Queue, function: Element) {
    function_queue.lock().await.push_back(function);
}