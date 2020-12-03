pub mod wasm;

#[no_mangle]
fn handle_data_upload_example(length: *const u8) {
    let length = length as usize;

    log(format!("Received byte array from JS with {} elements:", length).as_str());
    for i in 0..length {
        unsafe {
            log(format!("buffer[{}]: {}", i, wasm::get_buffer()[i]).as_str());
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

// main does not require #[no_mangle] to be called from WASM.
fn main() {
    log("Calling console.log() from Rust!");
}