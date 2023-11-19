#[allow(dead_code)]

pub mod govee {
    use std::time::Duration;
    // TODO recalculate
    pub const API_REQUEST_INTERVAL: Duration = Duration::from_millis(8730);
}

pub mod net {
    use std::net::{IpAddr, Ipv4Addr};
    pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    pub const PORT: u16 = 8080;
}