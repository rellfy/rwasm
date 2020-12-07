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