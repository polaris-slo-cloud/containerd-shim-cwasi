use std::thread::sleep;
use std::time::Duration;
use chrono;
use wasmedge_http_req::request;

fn main() {
    let mut writer = Vec::new(); //container for body of a response
    request::get("http://128.131.57.188:8080/", &mut writer).unwrap();
    let len = writer.len();
    println!("Received chunk of size: {} at {:?}", len, chrono::offset::Utc::now());

    let body_string = String::from_utf8(writer.to_vec()).unwrap();
    println!("After serialization at {:?}", chrono::offset::Utc::now());
    sleep(Duration::from_secs(5))
}
