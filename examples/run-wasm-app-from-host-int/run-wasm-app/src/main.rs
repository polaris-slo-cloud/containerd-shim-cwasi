use std::env;
use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    params, Vm, ImportObjectBuilder, WasmValue, Caller,
    error::HostFuncError, host_function
};

#[host_function]
fn my_add(_caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    // parse the first argument of WasmValue type
    println!("Inside host function");
    let first_argument = input[0].to_i32();
    let second_argument = input[1].to_i32();
    println!("first: {:?}", first_argument);
    println!("second: {:?}", second_argument);
    let result = first_argument + second_argument;
    Ok(vec![WasmValue::from_i32(result)])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let args_slice: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    //let wasm_app_file = std::env::args().nth(1).expect("Please specify a wasm file");
    let wasm_app_file = "target/wasm32-wasi/release/wasm-app.wasm";

    // create a config with the `wasi` option enabled
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;
    // create an import module
    let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32),i32>("real_add", my_add)?
        .build("shim_host_func")?;

    assert!(config.wasi_enabled());

    // create a VM with the config
    let mut vm = Vm::new(Some(config))?.register_import_module(import)?;


    vm.wasi_module()?.initialize(Some(args_slice), None, None);

    vm.register_module_from_file("wasm-app", &wasm_app_file)?
        .run_func(Some("wasm-app"), "_start", params!())?;

    Ok(())
}