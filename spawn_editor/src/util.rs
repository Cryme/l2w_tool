use std::fmt;

/// Takes two numbers as arguments and returns their quotient and remainder when using integer division.
fn divmod(x: u64, y: u64) -> (u64, u64) {
    let quotient = x / y;
    let remainder = x % y;
    (quotient, remainder)
}

#[derive(Debug, PartialEq)]
pub struct TimeHms {
    h: u64,
    m: u64,
    s: u64,
}

impl TimeHms {
    /// Converts a duration from a representation in seconds
    /// into a representation in hours, minutes and seconds.
    pub fn new(seconds: u64) -> TimeHms {
        let (m, s) = divmod(seconds, 60);
        let (h, m) = divmod(m, 60);

        TimeHms { h, m, s }
    }
}

impl fmt::Display for TimeHms {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            if self.h > 0 {
                format!("{}h ", self.h)
            } else {
                "".to_string()
            },
            if self.m > 0 {
                format!("{}m ", self.m)
            } else {
                "".to_string()
            },
            if self.s > 0 || (self.h == 0 && self.m == 0) {
                format!("{}s", self.m)
            } else {
                "".to_string()
            },
        )
    }
}
