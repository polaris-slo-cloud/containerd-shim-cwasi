use std::path::Path;
use std::thread::yield_now;
use wasmedge_sdk::{config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions}, dock::{Param, VmDock}, params, AsInstance, CallingFrame, Executor, ImportObjectBuilder, Module, Store, ValType, Vm, WasmValue};
use wasmedge_sdk::error::HostFuncError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let wasm_file = Path::new(&args[1]);
    let module = Module::from_file(None, wasm_file)?;
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;


    let mut vm = Vm::new(Some(config))?.register_module(None, module)?;
    let mut wasi_instance = vm.wasi_module()?;
    wasi_instance.initialize(None, None, None);
    let vm = VmDock::new(vm);


    // call func "say_ok" in wasm-lib.wasm: String -> Result<(u16, String), String>
    //let mut result:String = String::new();
    let mut result = Vec::new();
    let params = vec![Param::String(&args[2]), Param::String("127.0.0.1:8888")];
    let mut start:chrono::DateTime<chrono::Utc>=chrono::Utc::now();

    match vm.run_func("say_ok", params)? {
        Ok(mut res) =>{
            start = chrono::Utc::now();
            result = *res.pop().unwrap().downcast::<Vec<u8>>().unwrap();
        },
        Err(err) => {
            println!("Run bindgen -- say FAILED {}", err);
        }
    }


    println!("Start transfer of {} at {}", 0, start);


    /*println!("Received chunk of size: {} at {:?}", 0, chrono::offset::Utc::now());
    let input_str = std::str::from_utf8(&result).expect("Invalid UTF-8 sequence");
    //println!("After serialization at {:?}", chrono::offset::Utc::now());
    let params = vec![Param::String(input_str)];
    match vm.run_func("obfusticate", params)? {
        Ok(mut res) => {
            println!(
                "Run bindgen -- obfusticate: {}",
                res.pop().unwrap().downcast::<String>().unwrap()
            );
        }
        Err(err) => {
            println!("Run bindgen -- obfusticate FAILED {}", err);
        }
    }



    println!("Received chunk of size: {} at {:?}", 0, chrono::Utc::now());
    let input_str = std::str::from_utf8(&result).expect("Invalid UTF-8 sequence");
    let my_strings: [&str; 1] = [input_str];
    let my_vector: Vec<&str> = my_strings.to_vec();
    wasi_instance.initialize(
        Some(my_vector),
        Some(vec![]),
        Some(vec![]),
    );


    match vm.run_func("load_string", params!())? {
        Ok(mut res) => {
            println!(
                "Run bindgen -- load string: {}",
                res.pop().unwrap().downcast::<String>().unwrap()
            );
        }
        Err(err) => {
            println!("Run bindgen -- load string {}", err);
        }
    }

  */
    //let binding = b"Hello, WebAssembly!".to_vec();
    //println!("Data length: {}", result.len());
    //let params = vec![Param::VecU8(&result)];

    /*match vm.run_func("convert_bytes", params)? {
        Ok(mut res) => {
            let buf = *res.pop().unwrap().downcast::<Vec<u8>>().unwrap();
            // Convert the Vec<u8> to String
            //let result_string = String::from_utf8(buf).expect("Invalid UTF-8 sequence");
            //println!("Result string: {}", result_string);
        }
        Err(err) => {
            println!("Run bindgen -- obfusticate FAILED {}", err);
        }
    }

     */

    Ok(())
}
