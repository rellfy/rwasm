use std::collections::HashMap;
use std::collections::hash_map::Entry;

const BUFFER_SIZE: usize = 128_000;
static mut BUFFERS: Option<HashMap<u32, [u8; BUFFER_SIZE]>> = None;

pub fn get_buffer<'a>(id: u32) -> &'a [u8; BUFFER_SIZE] {
    let buffers;

    unsafe {
        if BUFFERS.is_none() {
            BUFFERS = Some(HashMap::new());
        }

        buffers = BUFFERS.as_mut().unwrap();
    }

    let buffer = match buffers.entry(id) {
        Entry::Occupied(o) => o.into_mut(),
        Entry::Vacant(v) => v.insert([0; BUFFER_SIZE])
    };

    buffer
}

pub fn get_buffer_slice(id: u32, size: usize) -> Vec<u8> {
    let buffer = get_buffer(id);
    (&buffer[0..size]).to_owned()
}

pub fn get_buffer_slice_as_string(id: u32, size: usize) -> String {
    let data = get_buffer_slice(id, size);
    String::from_utf8(data).unwrap()
}

pub fn delete_buffer(id: u32) -> Option<(u32, [u8; BUFFER_SIZE])> {
    let buffers;

    unsafe {
        if BUFFERS.is_none() {
            return None;
        }

        buffers = BUFFERS.as_mut().unwrap();
    }

    buffers.remove_entry(&id)
}

pub fn send_bytes(fn_name: &str, bytes: &[u8]) -> usize {
    let fn_name_bytes = fn_name.as_bytes();
    let mut data: Vec<u8> = vec![0; fn_name_bytes.len() + bytes.len() + 1];

    for i in 0..fn_name_bytes.len() {
        data[i] = fn_name_bytes[i];
    }

    for i in (fn_name_bytes.len()+1)..(fn_name_bytes.len()+1+bytes.len()) {
        data[i] = bytes[i - (fn_name_bytes.len()+1)];
    }

    unsafe {
        upload_bytes(data.as_ptr(), data.len())
    }
}

pub fn request_bytes(fn_name: &str, bytes: &[u8], buffer_id: u32) -> Vec<u8> {
    let string = format!("{}.{}", fn_name, buffer_id);
    let fn_name_with_buffer_id = string.as_str();
    let size = send_bytes(fn_name_with_buffer_id, bytes);
    get_buffer_slice(buffer_id, size)
}

pub fn request_string(fn_name: &str, bytes: &[u8], buffer_id: u32) -> String {
    String::from_utf8(request_bytes(fn_name, bytes, buffer_id)).unwrap()
}

pub fn get_bytes(fn_name: &str, buffer_id: u32) -> Vec<u8> {
    request_bytes(fn_name, &[], buffer_id)
}

pub fn get_string(fn_name: &str, buffer_id: u32) -> String {
    String::from_utf8(get_bytes(fn_name, buffer_id)).unwrap()
}

#[no_mangle]
extern "C" {
    pub fn upload_bytes(data: *const u8, length: usize) -> usize;
}

#[no_mangle]
fn get_buffer_pointer(id: u32) -> *const u8 {
    get_buffer(id).as_ptr()
}