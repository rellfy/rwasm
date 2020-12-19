pub mod wasync;
pub mod timer;
pub mod date;
mod error_hook;
mod wasm;

pub use {
    wasm::*,
    timer::TimerFuture
};

pub fn log(msg: &str) {
    wasm::send_bytes("console_log", msg.as_bytes());
}

pub fn set_error_hook() {
    std::panic::set_hook(Box::new(error_hook::hook));
}
