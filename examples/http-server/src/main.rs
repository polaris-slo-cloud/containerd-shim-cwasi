use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;
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
    let file_content = tokio::fs::read_to_string(file_path.to_string()).await?;
    // Bind the TCP listener to a port
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Listening on :8080");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let file_content = file_content.clone();


        let mut buf = [0; 1024];

        // Read the incoming data (simulate simple request)
        match socket.read(&mut buf).await {
            Ok(_) => {
                // Respond with a file
                if let Err(e) = serve_file(&mut socket, file_content).await {
                    eprintln!("Error serving file: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
            }
        }


        // Break the loop after the first connection
        println!("Shutting down the server after the first connection.");
        break;
    }
    Ok(())
}

// Function to serve the file to the client
async fn serve_file(socket: &mut tokio::net::TcpStream, file_buffer: String) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = chrono::offset::Utc::now();
    println!("Start transfer at {:?}", start_time);
    // Prepare a simple HTTP response header
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\n\r\n"
    );

    // Write the header to the socket
    socket.write_all(header.as_bytes()).await?;

    let total_sent = header.len() + file_buffer.len(); // Total bytes to be sent
    let start_time = chrono::offset::Utc::now();

    // Write the entire file buffer to the socket
    socket.write_all(&file_buffer.as_bytes()).await?;

    println!(
        "Overall sent {} bytes at {:?}",
        total_sent, start_time
    );

    Ok(())
}