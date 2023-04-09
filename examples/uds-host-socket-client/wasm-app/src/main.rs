pub fn add(left: i32, right: i32) -> i32 {
    println!("before real_add in wasm app");
    unsafe { shim_host_func::real_add(left, right) }
}

pub mod shim_host_func {
    #[link(wasm_import_module = "shim_host_func")]
    extern "C" {
        pub fn real_add(x: i32, y: i32) -> i32;
    }
}

#[no_mangle]
pub fn cwasi_function() -> i32 {
    println!("Greetings from wasm-app!");
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);

    let num1: i32 = args[1].parse().unwrap();
    let num2: i32 = args[2].parse().unwrap();

    let result = add(num1,num2);
    println!("Result inside wasm app{}",result);
    return result;
}

fn main(){
    println!("main end {}",cwasi_function());
}