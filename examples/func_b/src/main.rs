use std::time::Instant;
use chrono::{Utc};

fn main(){
    //let now = format!("{:?}", SystemTime::now());
    let now = Instant::now();
    println!("Greetings from func_b! at {:?}",now);
    cwasi_function();
}



#[no_mangle]
pub fn cwasi_function() -> i64 {
    let start = Utc::now();
    let _args: Vec<String> = std::env::args().collect();
    let data_loaded = Utc::now();
    println!("Wasm B started at {}",start);
    println!("Args read at {}",data_loaded);
    let nanos = data_loaded.timestamp_nanos_opt().unwrap();
    println!("Wasm B finished result {} at {}",Utc::now(),nanos);
    return nanos;
}
