pub fn add(arg1: String, arg2: String) {
    println!("before real_add in wasm app");
    let arg1_input_bytes = arg1.as_bytes();
    let arg1_len = arg1_input_bytes.len() as i32;
    let arg1_ptr = arg1_input_bytes.as_ptr() ;
    let arg1_ptr_i32 = arg1_ptr as i32;

    let arg2_input_bytes = arg2.as_bytes();
    let arg2_len = arg2_input_bytes.len() as i32;
    let arg2_ptr = arg2_input_bytes.as_ptr();
    let arg2_ptr_i32 = arg2_ptr as i32;
    //let response:String;
    unsafe {
        let response_length =shim_host_func::real_add(arg1_ptr_i32,arg1_len);
        println!("res len {:?} ",response_length);
        let bytes = std::slice::from_raw_parts(arg1_ptr, response_length as usize);
        let response = String::from_utf8_lossy(bytes).to_string();
        println!("response string {:?} ",response);
    }
    //return response;
}

pub mod shim_host_func {
    #[link(wasm_import_module = "shim_host_func")]
    extern "C" {
        pub fn real_add(arg1_ptr: i32, arg1_len: i32) -> i32;
    }
}

#[no_mangle]
pub fn cwasi_function() -> i32 {
    println!("Greetings from wasm-app!");
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);

    add(args[1].clone(),args[2].clone());
    //println!("Result inside wasm app");

    let num:i32 = 5;
    return num;
}


fn main(){
    let result = cwasi_function();
    //println!("main result {}",result);
   // println!("Wasm app finished");

}