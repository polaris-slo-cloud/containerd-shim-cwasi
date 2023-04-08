use std::env;

fn main() {
    println!("hello");
    let first_arg = env::args().nth(1);
    match first_arg {
        Some(arg) => println!("First argument: {}", arg),
        None => println!("No arguments found"),
    }
}