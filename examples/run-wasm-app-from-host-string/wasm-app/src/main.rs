use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};
use std::slice;
use std::alloc::{alloc, Layout};
use std::error::Error;
use wasmedge_http_req::request;

fn main() {
    println!("Greetings from wasm-app!");
    //let result= cwasi_function();

}



#[no_mangle]
pub fn cwasi_function() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?} read at {}", args,chrono::offset::Utc::now());
    //let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
    //println!("Value of STORAGE_IP: {}", storage_ip);
    let storage_ip = "127.0.0.1:8888";
    println!("Downloading file started at {}",chrono::offset::Utc::now());
    let file:String = args[2].parse().unwrap();
    let mut writer = Vec::new(); //container for body of a response
    let res = request::get("http://".to_owned()+&storage_ip+ &"/files/".to_owned()+&file, &mut writer).unwrap();
    println!("Downloading finished at {}",chrono::offset::Utc::now());
    let response_string = unsafe { &std::str::from_utf8_unchecked(&writer)};
    println!("Data copied to string at {}",chrono::offset::Utc::now());
    //println!("GET");
    println!("Response Status: {} {}", res.status_code(), res.reason());
    add(response_string);
    //println!("Result inside wasm app");

    let num:i32 = 5;
    return num;
}


#[no_mangle]
pub extern fn allocate_wasm(size: usize) -> *mut u8 {
    // Allocate memory with the requested size using Rust's global allocator
    let layout = Layout::from_size_align(size, 1).expect("Invalid layout");
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        panic!("Failed to allocate memory!");
    }

    ptr
}

#[allow(unused)]
#[no_mangle]
pub extern fn deallocate(pointer: *mut c_void, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(pointer, 0, capacity);
    }
}


#[no_mangle]
pub extern fn hello_greet_wasm(ptr: i32, len: i32) {
    println!("Received ptr: {:?}", ptr);
    println!("Received len: {:?}", len);
    let ptr = ptr as *const u8;
    let bytes = unsafe { slice::from_raw_parts(ptr, len as usize).to_vec() };
    let subject_str = String::from_utf8(bytes.clone()).expect("Invalid UTF-8");
    println!("Received subject: {}", subject_str);

}


#[no_mangle]
pub fn fib(ptr:i32, len:i32) -> i32 {
    println!("start fib");
    unsafe {
        let bytes = std::slice::from_raw_parts(&ptr as *const i32 as *const u8, len as usize);
        let response = &std::str::from_utf8_unchecked(bytes);
        println!("response string {:?} ",response);
    }

    /*let input_bytes = "Hello World!".as_bytes();
    let len = input_bytes.len() as i32;
    let ptr = input_bytes.as_ptr();
    let ptr_i32 = input_bytes.as_ptr() as i32;

     */
    //return [ptr_i32,len].to_vec();
    return 5;
}

pub fn add(input_string: &str){
    //println!("Received string {:?}",input_string);
    let input_bytes = input_string.as_bytes();

    let len = input_bytes.len() as i32;
    //let ptr = input_bytes.as_ptr();
    let ptr_i32 = input_string.as_ptr() as i32;
    //println!("wasm lib add {:?}",ptr);
    //println!("wasm lib len {:?}",len);
    unsafe {
        println!("start transfer at {}",chrono::offset::Utc::now());
        let response_length =my_math_lib::real_add(ptr_i32,len);
        //println!("res len {:?} ",response_length);
        //let bytes = std::slice::from_raw_parts(ptr, response_length as usize);
        //let response = String::from_utf8_lossy(bytes).to_string();
        //println!("response string {:?} ",response);
    }

}

pub mod my_math_lib {
    #[link(wasm_import_module = "my_math_lib")]
    extern "C" {
        pub fn real_add(ptr: i32, len: i32) -> i32;
    }
}


