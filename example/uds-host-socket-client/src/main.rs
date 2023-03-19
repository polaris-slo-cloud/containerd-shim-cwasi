use std::io::{Read, Write};
use regex::Regex;
use std::os::unix::net::{UnixStream};
use anyhow::Error;
use wasmedge_sdk::{error::HostFuncError, host_function, Caller, ImportObjectBuilder, WasmValue,WasmVal, Vm, params};
use wasmedge_sdk::config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions};

#[host_function]
fn my_add_host(_caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    let a = input[0].to_i32();
    let b = input[1].to_i32();

    println!("Called from module fnA with input {} and {}",a, b);
    let c = connect_unix_socket(a+b).unwrap();
    let result = a + b + c;

    println!("Resume function A with result {} + {} + {} = {}",a,b,c,result);
    Ok(vec![WasmValue::from_i32(result)])
}

fn connect_unix_socket(input_fn_a:i32)-> Result<i32, Error> {
    let mut stream = UnixStream::connect("../uds-host-socket-server/my_socket.sock").unwrap();
    let input_fn_b = format!("Data input from fn A {} \n", input_fn_a);
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
        .build("my_math_lib")?;

    // create a new Vm with default config
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    let num1: i32 = 8;
    let num2: i32 = 13;
    let mut vm = Vm::new(Some(config))?;

    vm.wasi_module()?.initialize(None, None, None);

    let res = vm.register_import_module(import)?
    .register_module_from_file("extern", "app-fnA.wasm")?
    .run_func(Some("extern"), "add", params!(num1, num2))?;

    println!("Result = {}", res[0].to_i32());
    Ok(())
}
