use std::net::SocketAddr;

use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response};
use tokio::net::TcpListener;
use chrono;
use wasmedge_http_req::request;

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {

        (&Method::GET, "/") => {
            let args: Vec<String> = std::env::args().collect();
            println!("args: {:?}", args);
            let storage_ip = std::env::var("STORAGE_IP").expect("Error: STORAGE_URL not found");
            println!("Value of STORAGE_IP: {}", storage_ip);

            println!("Downloading file");

            let file:String = args[2].parse().unwrap();
            let mut writer = Vec::new(); //container for body of a response
            let res = request::get("http://".to_owned()+&storage_ip+ &"/files/".to_owned()+&file, &mut writer).unwrap();
            println!("Status: {} {}", res.status_code(), res.reason());
            let len = writer.len();
            println!("Start transfer of {} at {}", len, chrono::offset::Utc::now());
            Ok(Response::new(Body::from(writer)))
        },


        (&Method::POST, "/hello") => {
            let start = chrono::offset::Utc::now();
            let received = format!("Received at {:?}",start);
            let name = hyper::body::to_bytes(req.into_body()).await?;
            let name_string = String::from_utf8(name.to_vec()).unwrap();

            let answer = format!("{}{}", "Hello ".to_owned(), name_string);

            Ok(Response::new(Body::from(received)))
        }

        _ => {
            Ok(Response::new(Body::from("😡 try again")))
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = Http::new().serve_connection(stream, service_fn(echo)).await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}