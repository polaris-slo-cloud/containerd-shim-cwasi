use std::alloc::{alloc, Layout};
use std::slice;
use std::str;

fn main() {
    println!("Greetings from Alice!");
}

#[no_mangle]
pub fn hello(left: i32, right: i32) -> i32 {
    //unsafe { alice_lib::export_add(left, right) }
    println!("Hello from library!");
    return 10;
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
pub extern fn hello_greet(ptr: i32, len: i32) {

    let ptr = ptr as *const u8;
    //let raw_bytes = unsafe { str::from_raw_parts(ptr, len as usize) };
    //let raw_string = unsafe {String::from_raw_parts(ptr as *mut u8, len as usize, len as usize)};

    let string_vec = unsafe { Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize)};
    //println!("Received chunk of size: {} at {:?}", len, chrono::offset::Utc::now());
    let raw_string = unsafe { String::from_utf8_unchecked(string_vec) };

    //let subject_str = str::from_utf8(raw_bytes).expect("Invalid UTF-8");
    println!("After serialization at {:?}", chrono::offset::Utc::now());
    //println!("Received subject: {:?}", raw_string);

}