use std::env::args;
use chrono::Utc;
use wasmedge_http_req::request;
#[allow(unused_imports)]
use wasmedge_bindgen::*;
use wasmedge_bindgen_macro::*;
#[wasmedge_bindgen]
pub fn say_ok(file: String, storage_ip: String) -> Result<Vec<u8>, String> {
    //let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
    println!("Value of STORAGE_IP: {}", storage_ip);
    let response_string;
    let mut input_bytes = Vec::new();
    unsafe {
        println!("Downloading file started at {}",chrono::offset::Utc::now());
        let mut writer = Vec::new(); //container for body of a response
        let res = request::get("http://".to_owned()+&storage_ip+ &"/files/".to_owned()+&file, &mut writer).unwrap();
        println!("Downloading finished at {}",chrono::offset::Utc::now());
        response_string = std::str::from_utf8_unchecked(&writer).to_string();
        input_bytes = response_string.into_bytes();
        //println!("Start transfer of {} at {}", 0, chrono::offset::Utc::now());
    }
    return Ok(input_bytes);
}


#[wasmedge_bindgen]
pub fn obfusticate(s: String) -> String {
    println!("After serialization {} at {:?}",0, chrono::offset::Utc::now());
    /*println!("Received chunk of size: {} at {:?}", s.len(), chrono::offset::Utc::now());

    let buffer = s.into_bytes();
    let start = chrono::offset::Utc::now();
    let body_string = String::from_utf8(buffer.to_vec());
    let duration = chrono::offset::Utc::now() - start;
    let seconds = duration.num_seconds() as f64
        + duration.num_nanoseconds().unwrap() as f64 / 1_000_000_000.0;
    println!("serialization at {:?}", seconds);

     */
    "finished".to_string()
}


#[wasmedge_bindgen]
pub fn load_string() -> String {
    println!("After serialization {} at {:?}",0, chrono::offset::Utc::now());

    //let _args: Vec<String> = std::env::args().collect();
    //println!("Args read at {:?}",chrono::offset::Utc::now());

    /*println!("Received chunk of size: {} at {:?}", s.len(), chrono::offset::Utc::now());

    let buffer = s.into_bytes();
    let start = chrono::offset::Utc::now();
    let body_string = String::from_utf8(buffer.to_vec());
    let duration = chrono::offset::Utc::now() - start;
    let seconds = duration.num_seconds() as f64
        + duration.num_nanoseconds().unwrap() as f64 / 1_000_000_000.0;
    println!("serialization at {:?}", seconds);

     */
    "finished".to_string()
}


#[wasmedge_bindgen]
pub fn convert_bytes(input: Vec<u8>) -> Vec<u8> {
    // Convert Vec<u8> to a String
    println!("Received chunk of size: {} at {:?}", input.len(), chrono::offset::Utc::now());
    let input_str = String::from_utf8(input).expect("Invalid UTF-8 sequence");
    println!("After serialization at {:?}", chrono::offset::Utc::now());
    // Convert the string to uppercase
    let output_str = input_str.to_uppercase();

    // Convert the output string back to Vec<u8> and return it
    output_str.into_bytes()
}