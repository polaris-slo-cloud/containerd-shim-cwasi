pub fn add(left: i32, right: i32) -> i32 {
    println!("before real_add alice wasm");
    unsafe { my_math_lib::real_add(left, right) }
}

pub mod my_math_lib {
    #[link(wasm_import_module = "my_math_lib")]
    extern "C" {
        pub fn real_add(x: i32, y: i32) -> i32;
    }
}

fn main() {
    println!("Greetings from  alice wasm-app!");
    //let args: Vec<String> = std::env::args().collect();
    //println!("args: {:?}", args);

    //let num1: i32 = args[1].parse().unwrap();
    //let num2: i32 = args[2].parse().unwrap();

    let num1: i32 = 5;
    let num2: i32 = 7;

    println!("before calling my_math_lib");
    let result = add(num1,num2);
    println!("after calling my_math_lib {}",result);
}