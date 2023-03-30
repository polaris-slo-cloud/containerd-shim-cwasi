use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    params, Vm
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let args: Vec<String> = std::env::args().collect();
    //println!("args: {:?}", args);

    //let num1: i32 = args[1].parse().unwrap();
    //let num2: i32 = args[2].parse().unwrap();

    // create a new Vm with default config
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    // create a vm and register bob and alice wasm modules into the vm

    let mut vm = Vm::new(Some(config))?;

    vm.wasi_module()?.initialize(None, None, None);

    let vm = vm.register_module_from_file(
            "my_math_lib",
            "target/wasm32-wasi/release/bob_wasm_lib.wasm",
        )?
        .register_module_from_file("alice", "target/wasm32-wasi/release/alice-wasm-app.wasm")?;

    let _res = vm.run_func(Some("alice"), "_start", params!())?;
    println!("Finished");

    Ok(())
}
