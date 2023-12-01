pub mod govee_secrets;

/// timezone to use for timers
pub const TIMEZONE: chrono_tz::Tz = chrono_tz::Europe::Berlin;

pub mod govee {
    use std::time::Duration;
    /// this is the max api request rate.
    /// will reach daily rate limit if used more than 16h40min in a single day
    /// (calling PUT device state every 6s).
    /// this is independent for GET and PUT device state calls,
    /// but calling GET every 6s for 24h is not possible. if you plan to use
    /// GET and PUT increase this interval to > 8s or something.
    pub const API_REQUEST_INTERVAL: Duration = Duration::from_secs(6);
    /// how long a `set_state()` call usually takes
    pub const AVG_SET_STATE_DURATION: Duration = Duration::from_millis(500);
}

pub mod brightness {
    pub const DAY: u8 = 15;
    pub const NIGHT: u8 = 1;
}

pub mod colors {
    pub const NIGHTLAMP: (u8, u8, u8) = (255, 181, 128);
}

// f64 types for easier calculations
pub mod sunrise {
    pub mod hsv_color {
        pub const HUE: f64 = 25.0;
        pub mod saturation {
            pub const START: f64 = 0.8;
            pub const STOP: f64 = 0.55;
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
    pub const PORT: u16 = 9000;
}