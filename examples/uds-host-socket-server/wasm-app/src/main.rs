pub fn add(left: i32, right: i32) -> i32 {
    println!("before real_add in wasm app");
    return left + right;
}

#[no_mangle]
pub fn cwasi_function() -> i32 {
    println!("Greetings from wasm-app!");
    let args: Vec<String> = std::env::args().collect();
    println!("args: {:?}", args);

    //let num1: i32 = args[1].parse().unwrap();
    //let num2: i32 = args[2].parse().unwrap();

    let result = add(5 as i32,10 as i32);
    println!("Result inside wasm app{}",result);
    return result;
}

fn main(){
    println!("main end {}",cwasi_function());
}