mod error_hook;
mod wasm_async;
mod wasm;
mod wasm_executor;

pub use wasm::*;
pub use wasm_async::*;
use std::time::Duration;
use futures::executor::block_on;

pub fn log(msg: &str) {
    wasm::send_bytes("console_log", msg.as_bytes());
}

pub fn set_error_hook() {
    std::panic::set_hook(Box::new((error_hook::hook)));
}

extern "C" {
    pub fn now() -> f64;
}

pub mod date {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn now() -> f64 {
        use std::time::SystemTime;

        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|e| panic!(e));
        time.as_secs_f64()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn now() -> f64 {
        unsafe { crate::now() }
    }
}