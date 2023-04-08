use std::io::{BufRead, BufReader, Read, Write};
use regex::Regex;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use anyhow::Error;
use wasmedge_sdk::{params, Vm};

pub fn connect_unix_socket(input_fn_a:i32)-> Result<i32, Error> {
    //connect to socket
    let mut stream = UnixStream::connect("../uds-host-socket-server/my_socket.sock").unwrap();
    let input_fn_b = format!("Data input from fn A {} \n", input_fn_a);
    //write request in the socket
    stream.write_all(input_fn_b.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("{}", response);
    // This is only for logging
    let re = Regex::new(r"\D+").unwrap();
    let result = re.replace(&*response,"").to_string();
    println!("Closing socket function A with B result {}",result);
    let i: i32 = result.parse().unwrap();
    Ok(i)

}

pub fn create_server_socket(vm: Vm)-> Result<(), Box<dyn std::error::Error>>{
    let socket_path = Path::new("my_socket.sock");
    if socket_path.exists() {
        std::fs::remove_file(&socket_path).unwrap();
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!("Socket created successfully at {:?}", &socket_path);

    match listener.accept() {
        Ok((mut socket, _addr)) => {
            // Read data from the socket stream
            let mut reader = BufReader::new(socket.try_clone()?);
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(_) => {
                    let client_input = line.trim();
                    println!("Received from client: {}", client_input);
                    // Send a response back to the client
                    reader.into_inner();
                    // Call function Code here
                    let result = call_vm_with_input(vm,client_input).unwrap();
                    let client_response = format!("hello world from from fnB socket server. Result from Module B : {}",result);
                    socket.write_all(client_response.as_bytes())?;

                }
                Err(err) => eprintln!("Error reading line: {}", err),
            }
        },
        Err(e) => println!("accept function failed: {:?}", e),
    }

    Ok(())
}

fn call_vm_with_input(mut vm: Vm, input: &str) -> Result<i32, Box<dyn std::error::Error>>{
    // create a new Vm with default config
    let re = Regex::new(r"\D+").unwrap();
    let num1: i32 = re.replace(&*input,"").to_string().parse().unwrap();
    let num2: i32 = 15;
    //let res = vm.run_func(Some("main"), "real_add", params!(num1,num2))?;

    println!("setting up wasi");
    let my_strings: [&str; 2] = [&num1.to_string(), &num2.to_string()];
    let my_vector: Vec<&str> = my_strings.to_vec();

    // Set new arguments on the wasi instance
    let mut wasi_instance = vm.wasi_module()?;
    wasi_instance.initialize(
        Some(my_vector),
        Some(vec![]),
        Some(vec![]),
    );
    let res = vm.run_func(Some("main"), "real_add", params!())?;
    let result = res[0].to_i32();
    println!("FnB Shim Finished. Result from moduleB: {}",result);

    Ok(result)
}