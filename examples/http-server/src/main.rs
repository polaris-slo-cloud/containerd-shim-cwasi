use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;
use tokio::fs::File;
use std::sync::Arc;
use chrono::SecondsFormat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the file path from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = Arc::new(args[1].clone());

    // Bind the TCP listener to a port
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Listening on :8080");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let file_path = file_path.clone();

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // Read the incoming data (simulate simple request)
            match socket.read(&mut buf).await {
                Ok(n) if n == 0 => return, // No data
                Ok(_) => {
                    // Respond with a file
                    if let Err(e) = serve_file(&mut socket, &file_path).await {
                        eprintln!("Error serving file: {:?}", e);
                    }
                }
                Err(e) => {
                    eprintln!("failed to read from socket; err = {:?}", e);
                }
            }
        });
    }
}

// Function to serve the file to the client
async fn serve_file(socket: &mut tokio::net::TcpStream, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Open the file asynchronously
    let mut file = File::open(file_path).await?;

    // Prepare a simple HTTP response header
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n"
    );

    socket.write_all(header.as_bytes()).await?;

    let mut total_sent = header.len(); // Start with the header length
    let start_time = chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);

    // Read the file in chunks and send it to the client
    let mut buf = [0; 1024];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        socket.write_all(&buf[0..n]).await?;
        total_sent += n; // Accumulate the size of each chunk sent
    }

    println!(
        "Overall sent {} bytes at {:?} ",
        total_sent, start_time
    );

    Ok(())
}