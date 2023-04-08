mod redis_utils;
mod message;

use uuid::Uuid;
use std::io::{Read, Write};
use regex::Regex;
use std::os::unix::net::{UnixStream};
use anyhow::Error;
use wasmedge_sdk::{error::HostFuncError, host_function, Caller, ImportObjectBuilder, WasmValue,WasmVal, Vm, params};
use wasmedge_sdk::config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions};

#[host_function]
fn my_add_host(_caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    let fn_id = input[0].to_i32();
    let fn_id_str = fn_id.to_string();

    let fn_input = input[1].to_i32();
    let ext_fn_result:i32;
    println!("Host func. From main app args {} {}",fn_id,fn_input);
    let hostname = hostname::get().unwrap();
    let hostname_str = hostname.to_str().unwrap();
    //check if the function is running locally
    if redis_utils::read(fn_id_str.as_str()).eq_ignore_ascii_case(hostname_str) {
        println!("Called from module fnA with input {} and {}",fn_id, fn_input);
        ext_fn_result = connect_unix_socket(fn_id+fn_input).unwrap();
    } else {
        ext_fn_result = connect_to_queue(fn_id, fn_input);
    }

    let result = fn_id + fn_input + ext_fn_result;

    println!("Resume function A with result {} + {} + {} = {}",fn_id,fn_input,ext_fn_result,result);
    Ok(vec![WasmValue::from_i32(result)])
}

fn connect_to_queue(fn_id :i32, fn_target_input:i32) -> i32{
    let fn_target_id_str = fn_id.to_string();
    let fn_source_id = Uuid::new_v4().to_simple().to_string();
    let fn_source_id_copy = fn_source_id.clone();

    let _ = redis_utils::publish_message(
        message::Message::new(fn_source_id,
                              fn_target_id_str, fn_target_input));

    let result = redis_utils::subscribe(fn_source_id_copy.as_str());

    return result;
}

fn connect_unix_socket(input_fn_a:i32)-> Result<i32, Error> {
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


fn main() -> Result<(), Box<dyn std::error::Error>>{

    // create an import module
    let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32), i32>("real_add", my_add_host)?
        .build("shim_host_func")?;

    // create a new Vm with default config
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    let args: Vec<String> = std::env::args().collect();
    let args_slice: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut vm = Vm::new(Some(config))?;

    vm.wasi_module()?.initialize(Some(args_slice), None, None);

    let res = vm.register_import_module(import)?
    .register_module_from_file("main-app", "wasm-app/target/wasm32-wasi/release/wasm-app.wasm")?
        .run_func(Some("main-app"), "_start", params!())?;

    println!("Result = {}", res[0].to_i32());
    Ok(())
}
