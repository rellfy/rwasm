
pub static mut BUFFER: [u8; 1_000] = [0; 1_000];

#[no_mangle]
fn handle_buffer(length: *const u8) {
    let length = length as usize;

    log(format!("Received byte array from JS with {} elements:", length).as_str());
    for i in 0..length {
        unsafe {
            log(format!("buffer[{}]: {}", i, BUFFER[i]).as_str());
        }
    }
}

#[no_mangle]
fn get_buffer_pointer() -> *const u8 {
    unsafe {
        BUFFER.as_ptr()
    }
}

#[no_mangle]
extern "C" {
    pub fn console_log(msg: *const u8, length: usize);
}

#[no_mangle]
fn multiply(x: f32, y: f32) -> f32 {
    x * y
}

fn log(msg: &str) {
    unsafe {
        console_log(msg.as_ptr(), msg.len());
    }
}

// main does not require #[no_mangle] to be called from WASM.
fn main() {
    log("Calling console.log() from Rust!");
}