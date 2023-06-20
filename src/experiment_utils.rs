extern crate reqwest;
use std::io::Read;

pub fn download(file:String) -> String {

    let mut body = String::new();
    println!("Downloading file {} at {:?}",file,chrono::offset::Utc::now());
    let mut res = reqwest::blocking::get("http://192.168.0.221:8888/files/".to_owned()+&file).unwrap();
    println!("Download finished at {:?}",chrono::offset::Utc::now());
    let _ = res.read_to_string(&mut body).unwrap().to_string();
    println!("String copied at {:?}",chrono::offset::Utc::now());

    println!("Status: {}", res.status());

    return body;
}