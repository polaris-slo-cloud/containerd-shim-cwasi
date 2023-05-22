use std::time::Instant;

fn main() {
    //let now = format!("{:?}", SystemTime::now());
    let now = Instant::now();
    println!("Greetings from func_b! at {:?}",now);
    cwasi_function();
}


#[no_mangle]
pub fn cwasi_function() -> i32 {
    println!("Wasm B started at {}",chrono::offset::Utc::now());
    let args: Vec<String> = std::env::args().collect();

    println!("args read at {}", chrono::offset::Utc::now());
    let input:String = args[2].parse().unwrap();

    //process_response(response_string);
    println!("input {:?}",input);
    let result:i32 = 5;
    println!("Wasm B finished at {}",chrono::offset::Utc::now());
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