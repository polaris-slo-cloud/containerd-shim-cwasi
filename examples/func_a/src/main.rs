use wasmedge_http_req::request;
use serde_json::{Result, Value};
use chrono::{DateTime, Utc,Duration};

fn main() {
   println!("Greetings from func_a {}",chrono::offset::Utc::now());
   cwasi_function();
}


#[no_mangle]
pub fn cwasi_function() -> i32 {
    unsafe {
        let args: Vec<String> = std::env::args().collect();
        println!("args: {:?} read at {}", args,chrono::offset::Utc::now());
        let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
        println!("Value of STORAGE_IP: {}", storage_ip);

        println!("Downloading file started at {}",chrono::offset::Utc::now());
        let file:String = args[2].parse().unwrap();
        let mut writer = Vec::new(); //container for body of a response
        let res = request::get("http://".to_owned()+&storage_ip+ &"/files/".to_owned()+&file, &mut writer).unwrap();
        println!("Downloading finished at {}",chrono::offset::Utc::now());
        let response_string = &std::str::from_utf8_unchecked(&writer);
        println!("Data copied to string at {}",chrono::offset::Utc::now());
        //println!("GET");
        println!("Response Status: {} {}", res.status_code(), res.reason());
        //println!("Headers {}", res.headers());

        //process_response(response_string);
        let index: i32 = std::env::var("FUNCTIONS_NUM").expect("Error: FUNCTIONS_NUM not found").parse().unwrap();
        println!("Value of FUNCTIONS_NUM: {}", index);
        let mut duration:Duration = Duration::seconds(0);
        let start = chrono::offset::Utc::now();
        for _ in 1..index {
            let response_b=process_response(response_string.clone()).replace("Received at ", "").replace("\n","");
            let datetime = DateTime::parse_from_rfc3339(&response_b)
                .unwrap_or_else(|err| panic!("Failed to parse date string: {}", err));

            // Convert the DateTime to the Utc timezone
            let datetime_utc: DateTime<Utc> = datetime.into();

            // Extract the date
            let duration_b= datetime_utc - start ;
            duration = duration+duration_b;
        }
        println!("Result  {} sent, Duration {} ms", index,duration.to_owned().num_milliseconds());
        let seconds = duration.num_microseconds().unwrap() as f64/1000000 as f64;
        let throughput = index as f64/ seconds as f64;
        println!("throughput: {}", throughput);

        let result:i32 = 5;
        return result;
    }
}

pub fn process_response(input_string: &str)-> String{
    println!("Process response ");
    let full_payload = "{\"source_channel\":\"func_a.wasm\",\"target_channel\":\"func_b.wasm\",\"payload\":\"".to_owned() +input_string+"\"}";
    let input_bytes = full_payload.as_bytes();
    let len = input_bytes.len() as i32;
    let ptr = input_bytes.as_ptr();
    let ptr_i32 = input_bytes.as_ptr() as i32;
    //println!("input pointer {:?} ",ptr);
    //println!("input length {:?} ",len);

    unsafe {
        println!("Call external func at {}",chrono::offset::Utc::now());
        let response_length =cwasi_export::func_connect(ptr_i32,len);
        println!("response from ext call received len {:?} at {}",response_length,chrono::offset::Utc::now());
        let bytes = std::slice::from_raw_parts(ptr, response_length as usize);
        println!("After bytes slice {}",chrono::offset::Utc::now());
        let response = &std::str::from_utf8_unchecked(bytes);
        println!("response string {:?} ",response);
        return response.to_string();
    }

}

pub mod cwasi_export {
    #[link(wasm_import_module = "cwasi_export")]
    extern "C" {
        pub fn func_connect(ptr: i32, len: i32) -> i32;
    }
}