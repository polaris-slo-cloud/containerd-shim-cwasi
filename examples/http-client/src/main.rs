use std::env;
use std::io::Read;
use bytes::Bytes;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    pretty_env_logger::init();

    // Some simple CLI args requirements...
    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return Ok(());
        }
    };

    // Parse the URL and make sure it's HTTP
    let url = url.parse::<reqwest::Url>().unwrap();
    if url.scheme() != "http" {
        println!("This example only works with 'http' URLs.");
        return Ok(());
    }

    fetch_url(url)
}

fn fetch_url(url: reqwest::Url) -> Result<()> {
    // Create a synchronous HTTP client
    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("Host", HeaderValue::from_str("web-server.default.svc.cluster.local").unwrap());
    // Make a GET request to the specified URL
    let res = client.get(url).headers(headers).send()?;

    // Check for success
    if res.status().is_success() {
        //println!("Request succeeded");

        // Accumulate response data in Bytes
        let mut buffer = Bytes::new();
        let body = res.bytes()?;
        buffer = body;

        // Print the size of the received data
        let start_time = chrono::offset::Utc::now();
        println!("Received chunk of size: {} at {:?}", buffer.len(), start_time);

        let body_string = String::from_utf8(buffer.to_vec())?;
        println!("After serialization at {:?}", chrono::offset::Utc::now());
    } else {
        println!("Request failed with status: {}", res.status());
    }

    Ok(())
}