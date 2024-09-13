use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{AsRawFd};
use libc::{vmsplice, iovec, SPLICE_F_MOVE, splice};
use std::{ptr, io, env};
use std::fs::File;
use std::io::Read;
use chrono::SecondsFormat;

fn handle_client(stream: TcpStream, file_path: &str) -> io::Result<()> {
    // Read the file into memory (Vec<u8>) using the provided file path
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let data_len = data.len();
    println!("Data size {} bytes", data_len);

    // Create a pipe
    let mut pipefd: [libc::c_int; 2] = [0; 2];
    unsafe {
        if libc::pipe(pipefd.as_mut_ptr()) == -1 {
            return Err(io::Error::last_os_error());
        }
    }

    let socket_fd = stream.as_raw_fd();
    let mut total_sent = 0;
    let chunk_size = 65536; // Use 64 KB chunks for transfer

    let start = chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    println!("start transfer at {:?}", start);
    // Transfer the in-memory data using vmsplice and splice
    while total_sent < data_len {
        let bytes_left = data_len - total_sent;
        let write_size = std::cmp::min(chunk_size, bytes_left);

        // Create the iovec structure to reference the in-memory data
        let iovec = iovec {
            iov_base: unsafe { data.as_ptr().add(total_sent) as *mut libc::c_void },
            iov_len: write_size,
        };
        // Use vmsplice to map the in-memory data to the pipe
        let n_written = unsafe {
            vmsplice(pipefd[1], &iovec as *const iovec, 1, SPLICE_F_MOVE)
        };

        if n_written == -1 {
            return Err(io::Error::last_os_error());
        }

        // Now splice the data from the pipe to the socket
        let mut total_transferred = 0;
        while total_transferred < n_written as usize {
            let bytes_sent = unsafe {
                splice(
                    pipefd[0],             // Read end of the pipe
                    ptr::null_mut(),       // Offset in the pipe (null for current)
                    socket_fd,             // Socket descriptor (output)
                    ptr::null_mut(),       // Offset in the socket (null for current)
                    n_written as usize - total_transferred,  // Remaining bytes to send
                    SPLICE_F_MOVE,         // Move the data (zero-copy)
                )
            };

            if bytes_sent == -1 {
                return Err(io::Error::last_os_error());
            }

            total_transferred += bytes_sent as usize;
        }

        total_sent += n_written as usize;
        //println!("Sent {} bytes so far", total_sent);
    }

    // Close the pipe after transferring the data
    unsafe {
        libc::close(pipefd[0]);
        libc::close(pipefd[1]);
    }

    let end_time = chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    println!("Overall sent {} bytes at {:?}", total_sent, end_time);

    // Close the stream to signal the client that the transmission is complete
    stream.shutdown(std::net::Shutdown::Write)?;

    Ok(())
}

fn main() -> io::Result<()> {
    // Read the file path from the command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    // Start the TCP listener
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Server is listening on port 8080...");

    // Handle each incoming client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New client connected!");
                if let Err(e) = handle_client(stream, file_path) {
                    eprintln!("Error handling client: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}