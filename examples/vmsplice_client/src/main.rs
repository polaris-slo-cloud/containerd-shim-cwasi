use std::net::TcpStream;
use std::io::{self, Read};
use std::env;
use chrono::SecondsFormat;

fn main() -> io::Result<()> {
    // Read the IP address and port from command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <ip:port>", args[0]);
        std::process::exit(1);
    }

    let address = &args[1]; // IP address and port, e.g., "127.0.0.1:8080"

    // Connect to the provided IP address and port
    let mut stream = TcpStream::connect(address)?;
    println!("Connected to the server at {}", address);

    let mut buffer = Vec::new();

    // Read all the data from the server until the connection is closed
    let bytes_read = stream.read_to_end(&mut buffer)?;
    let end_time = chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    println!("Received {} bytes at {:?}", bytes_read, end_time);

    Ok(())
}