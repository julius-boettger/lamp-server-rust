#[allow(dead_code)]

pub mod govee {
    use std::time::Duration;
    // TODO recalculate
    pub const API_REQUEST_INTERVAL: Duration = Duration::from_millis(8730);
    /// how long a `set_state()` call usually takes
    pub const AVG_SET_STATE_DURATION: Duration = Duration::from_millis(500);
}

// f64 types for easier calculations
pub mod sunrise {
    pub mod hsv_color {
        pub const HUE: f64 = 25.0;
        pub mod saturation {
            pub const START: f64 = 0.8;
            pub const STOP: f64 = 0.45;
        }
        pub const VALUE: f64 = 1.0;
    }
    pub mod govee_brightness {
        pub const START: f64 = 1.0;
        pub const STOP: f64 = 100.0;
    }
}

pub mod net {
    use std::net::{IpAddr, Ipv4Addr};
    pub const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    pub const PORT: u16 = 8080;
}