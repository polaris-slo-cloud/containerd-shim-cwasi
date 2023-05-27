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
    println!("I dont care much about input atm ");

    let result:i32 = 5;
    println!("Wasm B finished at {}",chrono::offset::Utc::now());
    return result;
}
