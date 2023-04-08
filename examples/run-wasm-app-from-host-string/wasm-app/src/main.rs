fn main() {
    println!("Greetings from wasm-app!");
    add("test string from app");
}

pub fn add(input_string: &str){
    println!("Received string {:?}",input_string);
    let input_bytes = input_string.as_bytes();
    let len = input_bytes.len() as i32;
    let ptr = input_bytes.as_ptr() as i32;
    println!("wasm lib add {:?}",ptr);
    println!("wasm lib len {:?}",len);
    unsafe { my_math_lib::real_add(ptr,len) }
}


pub mod my_math_lib {
    #[link(wasm_import_module = "my_math_lib")]
    extern "C" {
        pub fn real_add(ptr: i32, len: i32);
    }
}