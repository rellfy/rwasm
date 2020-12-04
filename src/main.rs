pub mod wasm;

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

fn log(msg: &str) {
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

// Main does not require #[no_mangle] to be called from WASM.
fn main() {
    // Log something to the console.
    log("Calling console.log() from Rust!");

    // Make a string uppercase from JS and return it to Rust.
    make_string_uppercase_from_js("this will be uppercase!");
}