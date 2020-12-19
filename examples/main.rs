use {
    rwasm,
    std::time::Duration,
};

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
    rwasm::timer::TimerFuture::new(Duration::from_millis(250)).await;
    rwasm::log("Hello...");
    rwasm::timer::TimerFuture::new(Duration::from_secs(1)).await;
    rwasm::log("...world!");
}

// Main does not require #[no_mangle] to be called from WASM.
fn main() {
    rwasm::set_error_hook();

    // Log something to the console.
    rwasm::log("Calling console.log() from Rust!");

    // Make a string uppercase from JS and return it to Rust.
    make_string_uppercase_from_js("this will be uppercase!");

    // Start async method.
    rwasm::wasync::Executor::spawn(main_async());
}