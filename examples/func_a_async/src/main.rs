#![warn(rust_2018_idioms)]
use hyper::{body::HttpBody as _, Client};
use hyper::{Body, Method, Request};
// use tokio::io::{self, AsyncWriteExt as _};

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let url_str = "http://localhost:1234";
    let post_body_str = "hello wasmedge";
    println!("\nPOST and get result as string: {}", url_str);
    println!("with a POST body: {}", post_body_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    async_post_url_return_str(url, post_body_str.as_bytes()).await;


    /*let url_str = "http://localhost:1234";
    println!("\nGET as byte stream: {}", url_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    if url.scheme_str() != Some("http") {
        println!("This example only works with 'http' URLs.");
        return Ok(());
    }
    fetch_url(url).await?;
     */

    let url_str = "http://localhost:1234";
    println!("\nGET and get result as string: {}", url_str);
    let url = url_str.parse::<hyper::Uri>().unwrap();
    sync_fetch_url_return_str(url).await?;



    //
    Ok(())
}

async fn fetch_url(url: hyper::Uri) -> Result<()> {
    println!(" Fetch");
    let client = Client::new();
    let mut res = client.get(url).await?;

    println!("Response: {}", res.status());
    println!("Headers: {:#?}\n", res.headers());

    /*tokio::task::spawn(async move {
        if let Some(next) = res.data().await {
            println!("Enter thread");
            let chunk = next.unwrap();
            println!("Response {:#?}", chunk);
        }
    });

  /*
    // Stream the body, writing each chunk to stdout as we get it
    // (instead of buffering and printing at the end).
    while let Some(next) = res.data().await {
        let chunk = next?;
        println!("response fetch url{:#?}", chunk);
        // io::stdout().write_all(&chunk).await?;
    }


/*
    Ok(())
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


async fn async_post_url_return_str(url: hyper::Uri, post_body: &'static [u8]) -> Result<()> {
    let client = Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .body(Body::from(post_body))?;
    let mut res = client.request(req).await?;


    tokio::task::spawn(async move {
        let mut resp_data = Vec::new();
        if let Some(next) = res.data().await {
            println!("Enter thread");
            let chunk = next.unwrap();
            resp_data.extend_from_slice(&chunk);
            println!("Response {:#?}", chunk);
        }
        println!("response ASYNC POST fetch url {}", String::from_utf8_lossy(&resp_data));
    });


    Ok(())
}

