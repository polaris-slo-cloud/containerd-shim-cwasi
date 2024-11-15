use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    Vm, ImportObjectBuilder, WasmValue, Caller, params,
    error::HostFuncError, host_function,WasmVal
};
use std::sync::{Arc, Mutex};


#[host_function]
fn my_add(_caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    let size = input[1].to_i32() as u32;
    Ok(vec![WasmValue::from_i32(size as i32)])
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let wasm_app_file = std::env::args().nth(1).expect("Please specify a wasm file");
    //let wasm_app_file = "target/wasm32-wasi/release/wasm-app.wasm";
    let wasm_app_file = "target/wasm32-wasi/release/fanout-wasi.wasm";
    let alice_file = "target/wasm32-wasi/release/alice-wasm-lib.wasm";


    // create a Const global instance
    // create a config with the `wasi` option enabled
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;
    println!("WASM app created.{}",config.max_memory_pages());
    // create a VM with the config
    let mut vm = Vm::new(Some(config))?;

    let mut envs: Vec<(String, String)> = std::env::vars().collect();
    envs.push(("STORAGE_IP".to_string(), "127.0.0.1:8888".to_string()));
    envs.push(("NUM_TASKS".to_string(), format!("{}",100)));
    let env_vars: Vec<String> = envs
        .into_iter()
        .map(|(key, value)| format!("{}={}", key, value))  // Convert (key, value) into "KEY=VALUE"
        .collect();
    let env_slice: Vec<&str> = env_vars.iter().map(|s| s.as_str()).collect();
    let args: Vec<String> = std::env::args().collect();
    let args_slice: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    //vm.wasi_module()?.initialize(Some(args_slice), None, None);
    let mut wasi_instance = vm.wasi_module()?;
    //let names = wasi_instance.memory_names();
    wasi_instance.initialize(Some(args_slice), Some(env_slice), None);


    let vm_shared = Arc::new(Mutex::new(vm.clone()));

    let vm_shared_for_closure = Arc::clone(&vm_shared);

    let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32), i32>("real_add", move |caller, input| {
            // parse the first argument of WasmValue type
            println!("[+] external function called: real_add");

            let address = input[0].to_i32();
            let len = input[1].to_i32();
            let result = my_add(caller, input)?;

            let mut vm_locked = vm_shared_for_closure.lock().unwrap();

            let alice_instance = vm_locked.named_module("alice-lib").unwrap();
            let mut alice_memory = alice_instance.memory("memory").unwrap();
            let allocate = alice_instance.func("allocate").unwrap();
            let alloc_result = allocate.call(&mut *vm_locked, params!(len as i32)).unwrap();
            let greet_mem_addr = alloc_result[0].to_i32();

            let instance = vm_locked.named_module("wasm-app").unwrap();
            let main_memory = instance.memory("memory").unwrap();
            let payload = main_memory.read(address as u32, len as u32).expect("fail to get string");


            let _ = alice_memory.write(payload, greet_mem_addr as u32);
            let greet = alice_instance.func("hello_greet").unwrap();
            let _ = greet.call(&mut *vm_locked, params!(greet_mem_addr as i32, len)).unwrap();
            Ok(result) // Return the result of the function to match the expected type
        })?
        .build("my_math_lib")?;

    let mut vm =vm.register_import_module(import)?
        .register_module_from_file("wasm-app", &wasm_app_file)?
        .register_module_from_file("alice-lib", &alice_file)?;

    let instance = vm.named_module("wasm-app").unwrap();
    //Main wasm app
    let start = instance.func("_start").unwrap();
    let _ = start.call(&mut vm, params!()).unwrap();
    //Call to alice allocate mem
    Ok(())
}
