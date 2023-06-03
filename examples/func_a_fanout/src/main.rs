use hyper::{body::HttpBody as _, Client};
use hyper::{Body, Method, Request};
// use tokio::io::{self, AsyncWriteExt as _};

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use chrono::{DateTime, Utc,Duration};
static mut TOTAL_DURATION_FANOUT: i64 = 0;
static mut TOTAL_DURATION_FANIN: i64 = 0;

fn main() {
    println!("Greetings from func_a!");
    cwasi_function();
}

#[no_mangle]
pub fn cwasi_function() -> String {
    return download();
}

#[tokio::main(flavor = "current_thread")]
pub async fn download() -> String {

    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);
    let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
    println!("Value of STORAGE_IP: {}", storage_ip);
    let funcb_url = std::env::var("FUNCB_URL").expect("Error: FUNCB_URL not found");
    println!("Value of FUNCB_URL: {}", funcb_url);

    println!("Downloading file");
    let file:String = args[2].parse().unwrap();

    let url = "http://".to_owned()+&storage_ip+ &"/files/".to_owned()+file.as_str();
    //println!("Downloading file {}",url);
    let url_download = url.parse::<hyper::Uri>().unwrap();
    let response_string=sync_fetch_url_return_str(url_download).await.unwrap();

    let index: i32 = std::env::var("FUNCTIONS_NUM").expect("Error: FUNCTIONS_NUM not found").parse().unwrap();
    println!("Value of FUNCTIONS_NUM: {}", index);
    unsafe {
        for i in 0..index {
            process_response(response_string.clone(),funcb_url.clone()).await;
            //println!("Request  {} sent, Duration {} ms", i, TOTAL_DURATION_FANOUT.to_string());
        }
        let url_download = url.parse::<hyper::Uri>().unwrap();
        let _=sync_fetch_url_return_str(url_download).await.unwrap();
        let seconds = TOTAL_DURATION_FANOUT as f64/1000000 as f64;
        let throughput = index as f64/ seconds as f64;
        let seconds_fanin = TOTAL_DURATION_FANIN as f64/1000000 as f64;
        let throughput_fanin = index as f64/ seconds_fanin as f64;
        println!("throughput fan-out: {}", throughput);
        println!("throughput fan-in: {}", throughput_fanin);
        println!("End at {:?}",chrono::offset::Utc::now());
    }
    return String::from("Im finished");
}

pub async fn process_response(file: String, funcb_url:String) -> Duration {



    let url_str = format!("http://{}",&funcb_url);
    let post_body_str = "hello wasmedge";
    //println!("\nPOST and get result as string: {}", url_str);
    //println!("with a POST body: {}", post_body_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    async_post_url_return_str(url.clone(), file).await;

    return Duration::seconds(1);
}

async fn async_post_url_return_str(url: hyper::Uri, post_body: String) -> Result<()> {
    let client = Client::new();
    //let start = chrono::offset::Utc::now();
    //println!("Connecting at {:?}",start);
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .body(Body::from(post_body))?;
    let start_clone = chrono::offset::Utc::now();
    println!("ASYNC Connecting at {:?}",start_clone);
    let mut res = client.request(req).await?;

    tokio::task::spawn(async move {
        let mut resp_data = Vec::new();
        if let Some(next) = res.data().await {
            println!("Enter thread ");
            let chunk = next.unwrap();
            resp_data.extend_from_slice(&chunk);

        }
        let received:DateTime<Utc> = chrono::offset::Utc::now();
        println!("ASYNC received {} len {}",received,resp_data.len());
        let mut body:String = "".to_string();
        for line in String::from_utf8_lossy(&resp_data).lines(){
            body=line.to_string();
            break;
        }
        //println!("ASYNC {}",body);
        let date_string = body.trim().replace("Received at ", "").replace("\n","").replace("\"","");
        //println!("FANIN Connecting at {}",date_string);
        //println!("FANIN Received at {}",received);
        //println!("data string: {}", date_string);
        // Parse the date string into a DateTime object
        let datetime = DateTime::parse_from_rfc3339(&date_string)
            .unwrap_or_else(|err| panic!("Failed to parse date string: {}", err));

        // Convert the DateTime to the Utc timezone
        let datetime_utc: DateTime<Utc> = datetime.into();

        // Extract the date
        println!("FANOUT using end date: {} start date {}", datetime_utc,start_clone);
        println!("FANIN using end date: {} start date {}", received,datetime_utc);
        let duration_fanout = datetime_utc - start_clone ;
        let duration_fanin = received - datetime_utc ;
        unsafe {
            println!("FANIN func duration {}", (duration_fanin.num_microseconds().unwrap() as f64)/1000000 as f64);
            println!("FANOUT func duration {}", (duration_fanout.num_microseconds().unwrap() as f64)/1000000 as f64);
            TOTAL_DURATION_FANOUT = TOTAL_DURATION_FANOUT + duration_fanout.num_microseconds().unwrap();
            TOTAL_DURATION_FANIN = TOTAL_DURATION_FANIN + duration_fanin.num_microseconds().unwrap();
        }
        //println!("response ASYNC POST fetch url {}", body);
    });


    Ok(())
}

async fn sync_fetch_url_return_str(url: hyper::Uri) -> Result<String> {
    //println!("SYNC Fetch url and return string {}",chrono::offset::Utc::now());
    let client = Client::new();
    let mut res = client.get(url).await?;

    let mut resp_data = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next.unwrap();
        resp_data.extend_from_slice(&chunk);
    }

    //println!("response SYNC fetch url {}",res.status());

    Ok(String::from_utf8_lossy(&resp_data).to_string())
}

