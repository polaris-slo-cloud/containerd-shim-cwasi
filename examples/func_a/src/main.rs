use wasmedge_http_req::request;
use chrono::{Duration};
use std::sync::{Arc, Mutex};


fn main(){
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
        send_request(response_string.clone());
        /*let rt = Runtime::new().unwrap();
        let _guard = rt.enter();
        for i in 0..index {
            let start = chrono::offset::Utc::now();
            let duration_func_microsec =process_response(response_string.clone()).await.replace("Received from client at : ", "").replace("\n", "");
            let duration_b=chrono::Duration::microseconds(duration_func_microsec.parse::<i64>().unwrap());

            duration = duration+duration_b;
            let seconds = duration.num_microseconds().unwrap() as f64/1000000 as f64;
            let throughput = (i+1) as f64/ seconds as f64;
            println!("throughput: {} index {}", throughput,i);
        }
        println!("Result  {} sent, Duration {} ms", index,duration.to_owned().num_milliseconds());
        let seconds = duration.num_microseconds().unwrap() as f64/1000000 as f64;
        let throughput = index as f64/ seconds as f64;
        println!("throughput: {}", throughput);
        */
        let result:i32 = 5;
        return result;
    }
}

pub fn send_request(input_string: &str) -> String{
    println!("Process response ");
    let start = chrono::offset::Utc::now();
    //let full_payload = "{\"source_channel\":\"func_a.wasm\",\"target_channel\":\"func_b.wasm\",\"payload\":\"".to_owned() +input_string+"\",\"start\":\"" +&start.to_string()+"\"}";
    let full_payload = format!("task={} start {}",0, input_string);
    let input_bytes = full_payload.as_bytes();
    //let input_bytes = Arc::new(Mutex::new(full_payload.into_bytes()));
    //let len = input_bytes.lock().unwrap().len() as i32;
    let len = input_bytes.len() as i32;
    let ptr = input_bytes.as_ptr();
    //let ptr_i32 = input_bytes.as_ptr() as i32;
    let ptr_i32 = full_payload.as_ptr() as i32;
    //println!("input pointer {:?} ",ptr);
    //println!("input length {:?} ",len);

    unsafe {

    println!("Call external func at {}",chrono::offset::Utc::now());
    println!("start transfer at {}",chrono::offset::Utc::now());
    let response_length =cwasi_export::func_connect(ptr_i32,len);
    println!("response from ext call received len {:?} at {:?}",response_length,chrono::offset::Utc::now());
    let bytes = std::slice::from_raw_parts(ptr, response_length as usize);
    println!("After bytes slice {}",chrono::offset::Utc::now());
    let response = &std::str::from_utf8_unchecked(bytes);
    //println!("response string {:?} ",response);

    //let cloned_input_bytes = Arc::clone(&input_bytes).lock().unwrap().clone();


    /*let input_bytes_ptr = cloned_input_bytes.as_ptr() as i32;
    let cloned_input_bytes_len = cloned_input_bytes.len();
    tokio::task::spawn(async move {
        let cloned_input_bytes_ptr = cloned_input_bytes.as_ptr();
        if let response_length = async_func_connect(input_bytes_ptr,len).await{
            println!("Response length {}",response_length);
            println!("response from ext call received len {:?} at {:?}",response_length,chrono::offset::Utc::now());
            let bytes = std::slice::from_raw_parts(cloned_input_bytes_ptr, response_length as usize);
            println!("After bytes slice {}",chrono::offset::Utc::now());
            response = &std::str::from_utf8_unchecked(bytes);
        }
        println!("response string {:?} ",response);
    });

     */
    "finished".to_string()
}

}
pub fn async_func_connect(str_ptr: i32, str_len: i32) -> i32 {
    unsafe { cwasi_export::func_connect(str_ptr, str_len) }
}
pub mod cwasi_export {
    #[link(wasm_import_module = "cwasi_export")]
    extern "C" {
        pub fn func_connect(ptr: i32, len: i32) -> i32;
    }
}