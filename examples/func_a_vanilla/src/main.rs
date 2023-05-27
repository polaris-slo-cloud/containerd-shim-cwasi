use wasmedge_http_req::request;
use chrono;

fn main() {
    println!("Greetings from func_a!");
    cwasi_function();
}


#[no_mangle]
pub fn cwasi_function() -> String {

    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);
    let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
    println!("Value of STORAGE_IP: {}", storage_ip);

    println!("Downloading file");

    let file:String = args[2].parse().unwrap();
    let mut writer = Vec::new(); //container for body of a response
    let res = request::get("http://".to_owned()+&storage_ip+ &"/files/".to_owned()+&file, &mut writer).unwrap();

    println!("GET");
    println!("Status: {} {}", res.status_code(), res.reason());
    println!("Headers {}", res.headers());

    process_response(writer);
    //println!("{}",response_string);
    //return response_string.to_string();
    return String::from("Im finished");
}

pub fn process_response(input_string: Vec<u8>){
    //println!("Process response ");
    //let full_payload = "{\"source_channel\":\"func_a.wasm\",\"target_channel\":\"func_b.wasm\",\"payload\":\"".to_owned() +input_string+"\"}";
    /*let input_bytes = full_payload.as_bytes();
    let len = input_bytes.len() as i32;
    let ptr = input_bytes.as_ptr();
    let _ptr_i32 = input_bytes.as_ptr() as i32;
    println!("input pointer {:?} ",ptr);
    println!("input length {:?} ",len);
     */
    http_client(input_string);

}

pub fn http_client(request_body: Vec<u8>){

    let funcb_ip = std::env::var("FUNCB_IP").expect("Error: FUNCB_IP not found");
    println!("Value of FUNCB_IP: {}", funcb_ip);

    let mut writer = Vec::new(); //container for body of a response
    //const BODY: &[u8; 27] = b"field1=value1&field2=value2";
    // let res = request::post("https://httpbin.org/post", BODY, &mut writer).unwrap();
    // no https , no dns
    let start = chrono::offset::Utc::now();
    println!("Connecting at {:?}",start);
    let res = request::post("http://".to_owned() + &funcb_ip+":1234/hello", &*request_body, &mut writer).unwrap();
    unsafe {
        let res_body = &std::str::from_utf8_unchecked(&writer);
        println!("POST");
        println!("Status: {} {}", res.status_code(), res.reason());
        println!("Headers {}", res.headers());
        println!("length {}",res_body.len());
        println!("{}",res_body);
    }
}