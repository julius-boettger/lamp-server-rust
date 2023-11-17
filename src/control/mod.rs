pub mod web;

use crate::util;
use crate::res::constants;

/// never terminates
pub async fn main_loop() {
    loop {
        util::sleep(constants::govee::API_REQUEST_INTERVAL);
    }
}