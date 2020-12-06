use std::panic;
use backtrace::Backtrace;
use crate::wasm;

pub fn hook(info: &panic::PanicInfo) {
    let mut message = info.to_string();

    message.push_str("\n\nStack:\n\n");
    message.push_str(format!("{:?}", Backtrace::new()).as_str());
    message.push_str("\n\n");

    wasm::send_bytes("console_error", message.as_bytes());
}