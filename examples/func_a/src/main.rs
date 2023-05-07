use wasmedge_http_req::request;

fn main() {
   println!("Greetings from func_a!");
   cwasi_function();
}


#[no_mangle]
pub fn cwasi_function() -> i32 {

    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);

    println!("Downloading file");
    let file:String = args[2].parse().unwrap();
    let mut writer = Vec::new(); //container for body of a response
    let res = request::get("http://127.0.0.1:8080/files/".to_owned()+&file, &mut writer).unwrap();
    let response_string = &String::from_utf8_lossy(&writer);
    println!("GET");
    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Headers {}", res.headers());

    process_response(response_string);

    let result:i32 = 5;
    return result;
}

pub fn process_response(input_string: &str){
    println!("Process response ");
    let full_payload = "{\"source_channel\":\"func_a.wasm\",\"target_channel\":\"func_b.wasm\",\"payload\":\"".to_owned() +input_string+"\"}";
    let input_bytes = full_payload.as_bytes();
    let len = input_bytes.len() as i32;
    let ptr = input_bytes.as_ptr();
    let ptr_i32 = input_bytes.as_ptr() as i32;
    println!("input pointer {:?} ",ptr);
    println!("input length {:?} ",len);

    unsafe {
        let response_length =cwasi_export::func_connect(ptr_i32,len);
        println!("res len {:?} ",response_length);
        let bytes = std::slice::from_raw_parts(ptr, response_length as usize);
        let response = String::from_utf8_lossy(bytes).to_string();
        println!("response string {:?} ",response);
    }

}

pub mod cwasi_export {
    #[link(wasm_import_module = "cwasi_export")]
    extern "C" {
        pub fn func_connect(ptr: i32, len: i32) -> i32;
    }
}