use std::time::Instant;
use chrono::{Utc};

fn main(){
    //let now = format!("{:?}", SystemTime::now());
    let now = Instant::now();
    println!("Greetings from func_b! at {:?}",now);
    //cwasi_function();
}

#[no_mangle]
pub extern fn allocate(size: usize) -> i32 {
    // Grow the WebAssembly memory by the requested size and return the pointer
    let ptr = wasm_alloc(size);
    ptr as i32  // Return the pointer (as an offset within WebAssembly memory)
}

fn wasm_alloc(size: usize) -> *mut u8 {
    // Simulate memory allocation by creating a buffer with the requested size
    let mut buffer = Vec::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);  // Don't drop the buffer, just leak it
    ptr
}

#[no_mangle]
pub extern fn cwasi_function(ptr: i32, len: i32) {

    let ptr = ptr as *const u8;
    //let raw_bytes = unsafe { str::from_raw_parts(ptr, len as usize) };
    //let raw_string = unsafe {String::from_raw_parts(ptr as *mut u8, len as usize, len as usize)};

    let string_vec = unsafe { Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize)};
    println!("Received {} bytes at {}", len, Utc::now());
    let raw_string = unsafe { String::from_utf8_unchecked(string_vec) };

    //let subject_str = str::from_utf8(raw_bytes).expect("Invalid UTF-8");
    println!("After serialization at {:?}", Utc::now());
    //println!("Received subject: {:?}", raw_string);

}

#[no_mangle]
pub fn hello() -> i64 {
    let start = Utc::now();
    let _args: Vec<String> = std::env::args().collect();
    let data_loaded = Utc::now();
    println!("Wasm B started at {}",start);
    println!("Args read at {}",data_loaded);
    let nanos = data_loaded.timestamp_nanos_opt().unwrap();
    println!("Wasm B finished result {} at {}",Utc::now(),nanos);
    return nanos;
}
