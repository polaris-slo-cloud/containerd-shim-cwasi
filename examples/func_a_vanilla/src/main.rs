use wasmedge_http_req::request;
use chrono::{DateTime, Utc,Duration};

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

    let index: i32 = std::env::var("FUNCTIONS_NUM").expect("Error: FUNCTIONS_NUM not found").parse().unwrap();
    println!("Value of FUNCTIONS_NUM: {}", index);
    let mut duration:Duration = Duration::seconds(0);

    for i in 0..index {
        let start = chrono::offset::Utc::now();
        let response_b=process_response(writer.clone()).replace("Received at ", "").replace("\n","");
        let datetime = DateTime::parse_from_rfc3339(&response_b)
            .unwrap_or_else(|err| panic!("Failed to parse date string: {}", err));

        // Convert the DateTime to the Utc timezone
        let datetime_utc: DateTime<Utc> = datetime.into();

        // Extract the date
        let duration_b = datetime_utc - start ;
        duration = duration+duration_b;
        let seconds = duration.num_microseconds().unwrap() as f64/1000000 as f64;
        let throughput = (i+1) as f64/ seconds as f64;
        println!("throughput: {} index {}", throughput,i);
    }
    println!("Result  {} sent, Duration {} ms", index,duration.to_owned().num_milliseconds());
    let seconds = duration.num_microseconds().unwrap() as f64/1000000 as f64;
    let throughput = index as f64/ seconds as f64;
    println!("throughput: {}", throughput);

    return String::from("Im finished");
}

pub fn process_response(input_string: Vec<u8>)->String{

    return http_client(input_string);

}

pub fn http_client(request_body: Vec<u8>)->String{

    let funcb_ip = std::env::var("FUNCB_IP").expect("Error: FUNCB_IP not found");
    println!("Value of FUNCB_IP: {}", funcb_ip);

    let mut writer = Vec::new(); //container for body of a response

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
        return res_body.to_string();
    }
}