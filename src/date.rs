use std::time::Duration;

extern "C" {
    fn seconds_now() -> f64;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> Duration {
    use std::time::SystemTime;

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|e| panic!(e));

    time
}

#[cfg(target_arch = "wasm32")]
pub fn now() -> Duration {
    unsafe {
        Duration::from_secs_f64(crate::date::seconds_now())
    }
}