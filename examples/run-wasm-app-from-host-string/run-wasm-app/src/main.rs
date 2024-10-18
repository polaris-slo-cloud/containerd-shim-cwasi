use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    Vm, ImportObjectBuilder, WasmValue, Caller, params, Global, GlobalType, ValType, Mutability, Table, TableType, RefType,
    error::HostFuncError, host_function, AsInstance, MemoryType, Memory,WasmVal
};
use std::sync::LazyLock;
use wasmedge_sdk::dock::Param;
extern crate libc;
use libc::{getrusage, rusage, RUSAGE_SELF};
use std::sync::{Arc, Mutex};
use wasmedge_sdk::types::Val;



fn get_context_switches() -> (i64, i64) {
    let mut usage: rusage = unsafe { std::mem::zeroed() };
    unsafe {
        getrusage(RUSAGE_SELF, &mut usage);
    }
    (usage.ru_nvcsw as i64, usage.ru_nivcsw as i64)
}

#[host_function]
fn my_add(caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    //println!("Greetings from host func!");
    //let mut mem = caller.memory(0).unwrap();

    //let addr = input[0].to_i32() as u32;
    let size = input[1].to_i32() as u32;

    // Calculate and print the context switch differences
    //println!("data: {:?}", data);
    //let mem_size = mem.size();
    //println!("mem size: {:?}", mem_size);
    //let result = memory.grow(1);

    //let s = String::from("this is a string create to be written on the memory");
    //let bytes = s.as_bytes();
    //let len = bytes.len();
    //mem.write(bytes, addr);

    Ok(vec![WasmValue::from_i32(size as i32)])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {


    //let wasm_app_file = std::env::args().nth(1).expect("Please specify a wasm file");
    let wasm_app_file = "target/wasm32-wasi/release/wasm-app.wasm";
    let alice_file = "target/wasm32-wasi/release/alice-wasm-lib.wasm";


    // create a Const global instance
    // create a config with the `wasi` option enabled
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    // create a VM with the config
    let mut vm = Vm::new(Some(config))?;

    let args: Vec<String> = std::env::args().collect();
    let args_slice: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    //vm.wasi_module()?.initialize(Some(args_slice), None, None);
    let mut wasi_instance = vm.wasi_module()?;
    //let names = wasi_instance.memory_names();
    wasi_instance.initialize(Some(args_slice), None, None);

    let vm_clone = Arc::new(vm.clone());

    let address = Arc::new(Mutex::new(0));  // Shared mutable value wrapped in Arc<Mutex>
    let len = Arc::new(Mutex::new(0));      // Shared mutable value wrapped in Arc<Mutex>

    let address_clone = Arc::clone(&address); // Clone the Arc for use inside the closure
    let len_clone = Arc::clone(&len);

    let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32), i32>("real_add", move |caller, input| {
            // parse the first argument of WasmValue type
            {
                let mut addr_lock = address_clone.lock().unwrap();
                *addr_lock = input[0].to_i32();
            }
            {
                let mut len_lock = len_clone.lock().unwrap();
                *len_lock = input[1].to_i32();
            }
            let result = my_add(caller, input)?;

            Ok(result) // Return the result of the function to match the expected type
        })?
        .build("my_math_lib")?;

    let mut vm=vm
        .register_import_module(import)?
        .register_module_from_file("wasm-app", &wasm_app_file)?
        .register_module_from_file("alice-lib", &alice_file)?;

    let instance = vm.named_module("wasm-app").unwrap();
    let mut main_memory = instance.memory("memory").unwrap();


    //Main wasm app
    let start = instance.func("_start").unwrap();
    let ret = start.call(&mut vm, params!()).unwrap();
    //Call to alice allocate mem
    let len:i32 = *len.lock().unwrap();
    //load data from main app
    let payload = main_memory.read(*address.lock().unwrap() as u32, len as u32).expect("fail to get string");
    println!("Received chunk of size: {} at {:?}", len, chrono::offset::Utc::now());
    let alice_instance = vm.named_module("alice-lib").unwrap();
    let mut alice_memory = alice_instance.memory("memory").unwrap();
    let allocate = alice_instance.func("allocate").unwrap();
    let result = allocate.call(&mut vm, params!(len as i32)).unwrap();
    let greet_mem_addr = result[0].to_i32();
    //println!("result alice allocate {:?}",greet_mem_addr);

    //println!("Payload to write: {}", payload);
    //write to alice data
    let _ = alice_memory.write(payload, greet_mem_addr as u32);

    let greet = alice_instance.func("hello_greet").unwrap();
    let _ = greet.call(&mut vm, params!(greet_mem_addr as i32, len)).unwrap();


    Ok(())
}
