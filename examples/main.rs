use rwasm;
use std::time::Duration;
use futures::Future;
use std::pin::Pin;
use core::task::Poll;

#[no_mangle]
fn handle_data_upload_example(length: *const u8) {
    let length = length as usize;

    rwasm::log(format!("Received byte array from JS with {} elements:", length).as_str());
    for i in 0..length {
        unsafe {
            rwasm::log(format!("buffer[{}]: {}", i, rwasm::get_buffer(0)[i]).as_str());
        }
    }
}

#[no_mangle]
fn multiply(x: f32, y: f32) -> f32 {
    x * y
}

fn make_string_uppercase_from_js(string: &str) {
    let string = rwasm::request_string(
        "request_data_example",
        string.as_bytes(),
        55
    );
    rwasm::log(format!("Received requested string: {}", string).as_str());
}

async fn main_async() {
    rwasm::log("Hello...");
    wait_seconds(0.5).await;
    // rwasm::TimerFuture::new(Duration::from_secs(5)).await;
    rwasm::log("...world!");
}

pub fn wait_seconds(time: f32) -> rwasm::TimerDelayFuture {
    rwasm::TimerDelayFuture {
        start_time: rwasm::date::now(),
        time,
    }
}

#[derive(Debug, PartialEq)]
pub enum ExecState {
    RunOnce,
    Waiting,
}

pub fn poll<'a>(future: &mut Pin<Box<dyn Future<Output = ()> + 'a>>) -> bool {
    let mut futures_context = ExecState::RunOnce;
    let futures_context_ref: &mut _ = unsafe {
        std::mem::transmute(&mut futures_context)
    };

    matches!(future.as_mut().poll(futures_context_ref), Poll::Ready(_))
}

// Main does not require #[no_mangle] to be called from WASM.
fn main() {
    rwasm::set_error_hook();

    // Log something to the console.
    rwasm::log("Calling console.log() from Rust!");

    // Make a string uppercase from JS and return it to Rust.
    make_string_uppercase_from_js("this will be uppercase!");

    let mut future: Pin<Box<dyn Future<Output = ()>>> = Box::pin(main_async());
    let mut i = 0;

    loop {
        rwasm::log(format!("i: {}", i).as_str());
        let ready = poll(&mut future);

        if ready {
            break;
        }
    }

    // let (executor, spawner) = wasm_executor::new_executor_and_spawner();
    // spawner.spawn(main_async());
    // drop(spawner);
    // executor.run();

    // block_on(main_async());
}