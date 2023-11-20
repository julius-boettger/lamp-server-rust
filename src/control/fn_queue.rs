use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::control::govee::SetState;

pub type FunctionQueue = Arc<Mutex<VecDeque<Box<dyn Fn(&mut VecDeque<SetState>) -> () + Send>>>>;

/// call and then remove each function, starting from the front.
pub fn call(function_queue: &mut FunctionQueue, govee_queue: &mut VecDeque<SetState>) {
    // TODO implement function. lock mutex!
}