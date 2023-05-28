use std::time::Instant;

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {

        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "👋 Hello World 🌍",
        ))),


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
    //let now = format!("{:?}", SystemTime::now());
    let now = Instant::now();
    println!("Greetings from func_b! at {:?}",now);
    cwasi_function();

    let addr = SocketAddr::from(([0, 0, 0, 0], 1234));

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



#[no_mangle]
pub fn cwasi_function() -> i32 {
    println!("Wasm B started at {}",chrono::offset::Utc::now());
    println!("I dont care much about input atm ");

    let result:i32 = 5;
    println!("Wasm B finished at {}",chrono::offset::Utc::now());
    return result;
}
