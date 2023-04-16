use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    Vm, ImportObjectBuilder, WasmValue, Caller, params,
    error::HostFuncError, host_function
};

#[host_function]
fn my_add(caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    println!("Greetings from host func!");
    let mut mem = caller.memory(0).unwrap();

    // parse the first argument of WasmValue type
    println!("Inside host function");
    let addr = input[0].to_i32() as u32;
    let size = input[1].to_i32() as u32;
    //println!("addr: {:?}", addr);
    //println!("size: {:?}", size);
    let data = mem.read_string(addr, size).expect("fail to get string");
    println!("data: {:?}", data);
    let mem_size = mem.size();
    println!("mem size: {:?}", mem_size);
    //let result = memory.grow(1);


    let s = String::from("this is a string create to be written on the memory");
    let bytes = s.as_bytes();
    let len = bytes.len();
    mem.write(bytes, addr);

    Ok(vec![WasmValue::from_i32(len as i32)])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let wasm_app_file = std::env::args().nth(1).expect("Please specify a wasm file");
    let wasm_app_file = "target/wasm32-wasi/release/wasm-app.wasm";

    // create a config with the `wasi` option enabled
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;
    // create an import module
    let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32), i32>("real_add", my_add)?
        .build("my_math_lib")?;

    assert!(config.wasi_enabled());

    // create a VM with the config
    let mut vm = Vm::new(Some(config))?.register_import_module(import)?;

    let args: Vec<String> = std::env::args().collect();
    let args_slice: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    vm.wasi_module()?.initialize(Some(args_slice), None, None);

    let vm=vm.register_module_from_file("wasm-app", &wasm_app_file)?
        ;
    //.run_func(Some("wasm-app"), "_start", params!())?
    let ret = match vm.run_func(Some("wasm-app"), "_start", params!()) {
        Ok(ok) => std::process::exit(0),
        Err(_) => std::process::exit(137),
    };

    println!("result {:?}",ret);

    Ok(())
}
