fn main() {
    println!("Greetings from wasm-app!");
    let result= cwasi_function();
    println!("result: {:?}", result);
}

#[no_mangle]
pub fn cwasi_function() -> i32 {
    println!("Greetings from wasm-app!");
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);
    let function_name:&str = &args[1].clone();
    add(function_name);
    //println!("Result inside wasm app");

    let num:i32 = 5;
    return num;
}

pub fn add(input_string: &str){
    println!("Received string {:?}",input_string);
    let input_bytes = input_string.as_bytes();
    let len = input_bytes.len() as i32;
    let ptr = input_bytes.as_ptr();
    let ptr_i32 = input_bytes.as_ptr() as i32;
    //println!("wasm lib add {:?}",ptr);
    //println!("wasm lib len {:?}",len);
    unsafe {
        let response_length =shim_host_func::real_add(ptr_i32,len);
        println!("res len {:?} ",response_length);
        let bytes = std::slice::from_raw_parts(ptr, response_length as usize);
        let response = String::from_utf8_lossy(bytes).to_string();
        println!("response string {:?} ",response);
    }

}

pub mod shim_host_func {
    #[link(wasm_import_module = "shim_host_func")]
    extern "C" {
        pub fn real_add(ptr: i32, len: i32) -> i32;
    }
}
