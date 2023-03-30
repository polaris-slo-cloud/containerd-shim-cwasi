use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener};
use std::path::Path;
use regex::Regex;
use wasmedge_sdk::{Vm, params,WasmVal
};
use wasmedge_sdk::config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions};


fn create_vm_and_load_module(input: &str) -> Result<i32, Box<dyn std::error::Error>>{
    // create a new Vm with default config
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    let mut vm = Vm::new(Some(config))?;
    vm.wasi_module()?.initialize(None, None, None);
    let vm = vm.register_module_from_file(
        "main",
        "app_fnB.wasm",
    )?;
    let re = Regex::new(r"\D+").unwrap();
    let num1: i32 = re.replace(&*input,"").to_string().parse().unwrap();
    let num2: i32 = 15;
    let res = vm.run_func(Some("main"), "real_add", params!(num1,num2))?;
    let result = res[0].to_i32();
    println!("FnB Shim Finished. Result from moduleB: {}",result);

    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

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
                    let result = create_vm_and_load_module(client_input).unwrap();
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
