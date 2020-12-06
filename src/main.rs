pub mod wasm;
pub mod wasm_async;
mod error_hook;
mod wasm_executor;

use std::time::Duration;
use futures::executor::block_on;
use crate::wasm_async::TimerFuture;
use std::pin::Pin;
use std::future::Future;

#[no_mangle]
fn handle_data_upload_example(length: *const u8) {
    let length = length as usize;

    log(format!("Received byte array from JS with {} elements:", length).as_str());
    for i in 0..length {
        unsafe {
            log(format!("buffer[{}]: {}", i, wasm::get_buffer(0)[i]).as_str());
        }
    }
}

#[no_mangle]
fn multiply(x: f32, y: f32) -> f32 {
    x * y
}

pub fn log(msg: &str) {
    wasm::send_bytes("console_log", msg.as_bytes());
}

fn make_string_uppercase_from_js(string: &str) {
    let string = wasm::request_string(
        "request_data_example",
        string.as_bytes(),
        55
    );
    log(format!("Received requested string: {}", string).as_str());
}

async fn main_async() {
    log("Hello...");
    // TimerFuture::new(Duration::from_secs(1)).await;
    log("...world!");
}

// Main does not require #[no_mangle] to be called from WASM.
fn main() {
    std::panic::set_hook(Box::new((error_hook::hook)));

    // Log something to the console.
    log("Calling console.log() from Rust!");

    // Make a string uppercase from JS and return it to Rust.
    make_string_uppercase_from_js("this will be uppercase!");

    // let (executor, spawner) = wasm_executor::new_executor_and_spawner();

    loop {
        let mut future: Pin<Box<dyn Future<Output = ()> + 'static>> = Box::pin(main_async());
        let mut futures_context: Option<bool> = None;
        let futures_context_ref: &mut _ = unsafe { std::mem::transmute(&mut futures_context) };

        while !matches!(future.as_mut().poll(futures_context_ref), core::task::Poll::Ready(_)) { }
        break;
    }


    // spawner.spawn(main_async());

    // drop(spawner);

    // executor.run();

    // block_on(main_async());
}