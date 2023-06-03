use hyper::{body::HttpBody as _, Client};
use hyper::{Body, Method, Request};
use chrono::{Duration};
use tokio::task::JoinHandle;
use wasmedge_http_req::request;

// use tokio::io::{self, AsyncWriteExt as _};

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    println!("Greetings from func_a {}",chrono::offset::Utc::now());

    cwasi_function().await;
    Ok(())
}

#[no_mangle]
pub async fn cwasi_function() -> i32 {
    unsafe {
        let response_string="download";
        let index: i32 = std::env::var("FUNCTIONS_NUM").expect("Error: FUNCTIONS_NUM not found").parse().unwrap();
        println!("Value of FUNCTIONS_NUM: {}", index);
        let mut duration:Duration = Duration::seconds(0);
        for i in 0..index {
            let duration_func_microsec =process_response(response_string.clone()).await as i64;
            //let duration_b=chrono::Duration::microseconds(duration_func_microsec.parse::<i64>().unwrap());

            duration = duration+Duration::microseconds(duration_func_microsec);
            let seconds = duration.num_microseconds().unwrap() as f64/1000000 as f64;
            let throughput = (i+1) as f64/ seconds as f64;
            println!("throughput: {} index {}", throughput,i);
        }
        println!("Result  {} sent, Duration {} ms", index,duration.to_owned().num_milliseconds());
        let seconds = duration.num_microseconds().unwrap() as f64/1000000 as f64;
        let throughput = index as f64/ seconds as f64;
        println!("throughput: {}", throughput);

        let result:i32 = 5;
        return result;
    }
}

pub async fn process_response(input_string: &str)-> i32{
    println!("Process response ");
    let full_payload = "{\"source_channel\":\"func_a.wasm\",\"target_channel\":\"func_b.wasm\",\"payload\":\"".to_owned() +input_string+"\"}";

    let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_IP not found");
    println!("Value of STORAGE_IP: {}", storage_ip);
    let url_str = format!("http://{}",storage_ip);
    let post_body_str = "hello wasmedge";
    println!("\nPOST and get result as string: {}", url_str);
    println!("with a POST body: {}", post_body_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    let response=async_post_url_return_str(url, full_payload);

    let url = url_str.parse::<hyper::Uri>().unwrap();
    sync_fetch_url_return_str(url).await.unwrap();

    return response.await;

}


async fn async_post_url_return_str(url: hyper::Uri, post_body:String) -> i32 {
    let post_body=post_body.as_bytes().to_owned();
    let post_body_clone = post_body.to_owned();
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .body(Body::from(post_body_clone)).unwrap();

    let len = post_body.clone().len();
    //let cloned_input_bytes_ptr = post_body.clone().as_ptr();
    let cloned_input_bytes_ptr_i32 = post_body.as_ptr() as i32;


    println!("Before call ffi  ptr i32  {}",cloned_input_bytes_ptr_i32);
    let response_length=call_ffi_get_new_length(req, cloned_input_bytes_ptr_i32, len as i32).await.await.unwrap();
    //println!("After call ffi  ptr {:?}",cloned_input_bytes_ptr);
    //let bytes = std::slice::from_raw_parts(cloned_input_bytes_ptr, response_length as usize);
    //println!("After bytes slice {}",chrono::offset::Utc::now());
    //let response = std::str::from_utf8_unchecked(bytes).to_string();
    //println!("Response from thread {}",response);
    return response_length;



}

async fn call_ffi_get_new_length(req:Request<Body>, ptr:i32, len:i32)-> JoinHandle<i32>{
    let client = Client::new();
    let mut res = client.request(req).await.unwrap();
    let mut resp_data:Vec<u8> = Vec::new();
    tokio::task::spawn(async move {
        let mut response_length= len.clone();
        if let Some(_next) = res.data().await {
            unsafe {
                //let chunk = next.unwrap();
                //resp_data.extend_from_slice(&chunk);
                println!("ASYNC START Enter thread ptr {} at {}",ptr,chrono::offset::Utc::now());
                response_length=cwasi_export::func_connect(ptr,len);

                println!("Response length response {}",response_length);
                println!("response from ext call received len {:?} at {:?}",response_length,chrono::offset::Utc::now());
               //println!("Response {:#?}", chunk);
            }
        }
        println!("Response length return {}",response_length);
        return response_length;
        //println!("response ASYNC POST fetch url {}", String::from_utf8_lossy(&resp_data));
    })
}


pub mod cwasi_export {
    #[link(wasm_import_module = "cwasi_export")]
    extern "C" {
        pub fn func_connect(ptr: i32, len: i32) -> i32;
    }
}

async fn sync_fetch_url_return_str(url: hyper::Uri) -> Result<()> {
    println!("Fetch url and return string");
    let client = Client::new();
    let mut res = client.get(url).await?;

    let mut resp_data = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next?;
        resp_data.extend_from_slice(&chunk);
    }
    println!("response SYNC fetch url str{}", String::from_utf8_lossy(&resp_data));

    Ok(())
}