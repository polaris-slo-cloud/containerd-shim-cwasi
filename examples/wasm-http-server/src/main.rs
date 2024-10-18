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
            Ok(Response::new(Body::from("Hello World!")))
        },


        (&Method::POST, "/") => {
            let body = hyper::body::to_bytes(req.into_body()).await?;
            let body_string = String::from_utf8(body.to_vec()).unwrap();
            let answer = format!("{}{}", "Hello ".to_owned(), body_string);
            Ok(Response::new(Body::from(answer)))
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