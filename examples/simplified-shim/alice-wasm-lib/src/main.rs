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
    let string_vec = unsafe { Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize)};
    let received_chunk =chrono::offset::Utc::now();
    let raw_string = unsafe { String::from_utf8_unchecked(string_vec) };
    let after_serialization =chrono::offset::Utc::now();
    let task_id = extract_task_id(&raw_string).unwrap_or("0");
    println!("Received chunk of size: {} at {:?} for {}", len, received_chunk, task_id);
    println!("After serialization at {:?} for {}",after_serialization,task_id);
    //println!("Received subject: {:?}", raw_string);

}

// Function to extract the task ID from the string
fn extract_task_id(content_task: &str) -> Option<&str> {
    // Look for the substring "task=" and the position of " start"
    let task_prefix = "task=";
    let start_marker = " start";

    if let Some(start_idx) = content_task.find(task_prefix) {
        if let Some(end_idx) = content_task.find(start_marker) {
            // Get the slice of the task ID by slicing between the found indexes
            let task_id_start = start_idx + task_prefix.len(); // Skip "task="
            return Some(&content_task[task_id_start..end_idx]);
        }
    }

    None // Return None if extraction fails
}