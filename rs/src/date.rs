use std::time::Duration;

extern "C" {
    fn seconds_now() -> f64;
}

pub fn now() -> Duration {
    unsafe {
        Duration::from_secs_f64(crate::date::seconds_now())
    }
}