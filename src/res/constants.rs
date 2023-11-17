#[allow(dead_code)]

pub mod govee {
    // TODO recalculate
    /// in ms
    pub const API_REQUEST_INTERVAL: u64 = 8730;
}

pub mod net {
    use std::net::{IpAddr, Ipv4Addr};
    pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    pub const PORT: u16 = 8080;
}