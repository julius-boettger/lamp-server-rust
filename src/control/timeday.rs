#[derive(Debug)]
pub struct TimeDay {
    hour: u8,
    minute: u8,
    days: Vec<u8>
}

impl TimeDay {
    /// panics if at least one value is out of range. see setters for ranges.
    fn new(hour: u8, minute: u8, days: Vec<u8>) -> Self {
        let mut instance = TimeDay { hour: 0, minute: 0, days: vec![] };
        instance.set_hour(hour);
        instance.set_minute(minute);
        instance.set_days(days);
        instance
    }

    fn get_hour(&self) -> &u8 { &self.hour }
    fn get_minute(&self) -> &u8 { &self.minute }
    fn get_days(&self) -> &Vec<u8> { &self.days }

    /// 0 to 23. panics if value is out of range.
    fn set_hour(&mut self, hour: u8) {
        if hour > 23 {
            panic!("hour has to be <= 23, was {:?}", hour);
        }
        self.hour = hour;
    }

    /// 0 to 59. panics if value is out of range.
    fn set_minute(&mut self, minute: u8) {
        if minute > 59 {
            panic!("minute has to be <= 59, was {:?}", minute);
        }
        self.minute = minute;
    }

    /// panics if value is out of range or vector is empty. <br>
    /// 0 - monday <br>
    /// 1 - tuesday <br>
    /// 2 - wednesday <br>
    /// 3 - thursday <br>
    /// 4 - friday <br>
    /// 5 - saturday <br>
    /// 6 - sunday <br>
    fn set_days(&mut self, days: Vec<u8>) {
        if days.is_empty() {
            panic!("days must not be empty");
        }
        if days.iter().any(|&d| d > 6) {
            panic!("every day has to be <= 6, days were {:?}", days);
        }
        self.days = days;
    }

    /// can shift in both forwards and backwards in time
    fn shift_time(&self, hour_shift: i8, minute_shift: i8) -> TimeDay {
        let mut total_minutes: i16 = i16::from(self.minute ) + (i16::from(self.hour ) * 60);
        let     shift_minutes: i16 = i16::from(minute_shift) + (i16::from(hour_shift) * 60);
        total_minutes += shift_minutes;

        let mut days = self.days.clone();
        // shift days until total_minutes is positive
        while total_minutes < 0 {
            days = days.into_iter().map(|day|
                if day == 0 { 6 } else { day - 1 }
            ).collect();
            total_minutes += 60 * 24; // minutes in a day
        }
        // shift days until total_minutes is less than a day
        while total_minutes >= 60 * 24 {
            days = days.into_iter().map(|day|
                if day == 6 { 0 } else { day + 1 }
            ).collect();
            total_minutes -= 60 * 24; // minutes in a day
        }

        let hour = total_minutes / 60;
        let minute = total_minutes - (hour * 60);

        Self::new(hour as u8, minute as u8, days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_minute_shift() {
        let timeday = TimeDay::new(15, 20, vec![5]);
        let result = timeday.shift_time(0, 5);
        assert_eq!(result.get_days(), timeday.get_days());
        assert_eq!(result.get_hour(), timeday.get_hour());
        assert_eq!(*result.get_minute(), 25);
    }

    #[test]
    fn minute_shift() {
        let timeday = TimeDay::new(15, 20, vec![5]);
        let result = timeday.shift_time(0, 45);
        assert_eq!(result.get_days(), timeday.get_days());
        assert_eq!(*result.get_hour(), 16);
        assert_eq!(*result.get_minute(), 5);
    }

    #[test]
    fn midnight_minute_shift() {
        let timeday = TimeDay::new(0, 15, vec![0, 1]);
        let result = timeday.shift_time(0, -16);
        assert_eq!(*result.get_days(), vec![6, 0]);
        assert_eq!(*result.get_hour(), 23);
        assert_eq!(*result.get_minute(), 59);
    }

    #[test]
    fn simple_hour_shift() {
        let timeday = TimeDay::new(12, 5, vec![0, 1, 2, 3, 4, 5, 6]);
        let result = timeday.shift_time(-2, 0);
        assert_eq!(result.get_days(), timeday.get_days());
        assert_eq!(*result.get_hour(), 10);
        assert_eq!(result.get_minute(), timeday.get_minute());
    }

    #[test]
    fn midnight_hour_shift() {
        let timeday = TimeDay::new(12, 5, vec![2, 3, 5, 6]);
        let result = timeday.shift_time(-14, 0);
        assert_eq!(*result.get_days(), vec![1, 2, 4, 5]);
        assert_eq!(*result.get_hour(), 22);
        assert_eq!(result.get_minute(), timeday.get_minute());
    }

    #[test]
    fn complex_forward_shift() {
        let timeday = TimeDay::new(21, 46, vec![0, 2, 3, 6]);
        let result = timeday.shift_time(3 + 24, 14);
        assert_eq!(*result.get_days(), vec![2, 4, 5, 1]);
        assert_eq!(*result.get_hour(), 1);
        assert_eq!(*result.get_minute(), 0);
    }

    #[test]
    fn complex_backward_shift() {
        let timeday = TimeDay::new(3, 4, vec![0, 2, 3, 6]);
        let result = timeday.shift_time(-12 - 24, -4);
        assert_eq!(*result.get_days(), vec![5, 0, 1, 4]);
        assert_eq!(*result.get_hour(), 15);
        assert_eq!(*result.get_minute(), 0);
    }
}