#[allow(dead_code)]

/// in ms
pub const DELAY: u16 = 8730;

// networking
use std::net::{IpAddr, Ipv4Addr};
pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub const PORT: u16 = 8080;