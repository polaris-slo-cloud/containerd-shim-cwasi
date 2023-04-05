use wasmedge_sdk::{Caller, WasmValue,host_function};
use wasmedge_sdk::error::HostFuncError;
use regex::Regex;
use std::os::unix::net::{UnixStream};
use std::io::{Read, Write};
use crate::socket_utils;


#[host_function]
pub fn func_connect(_caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    let fn_id = input[0].to_i32();
    let fn_id_str = fn_id.to_string();

    let fn_input = input[1].to_i32();
    let ext_fn_result:i32;
    let hostname = hostname::get().unwrap();
    let hostname_str = hostname.to_str().unwrap();
    //check if the function is running locally
    /*if redis_utils::read(fn_id_str.as_str()).eq_ignore_ascii_case(hostname_str) {
        println!("Called from module fnA with input {} and {}",fn_id, fn_input);
        ext_fn_result = connect_unix_socket(fn_id+fn_input).unwrap();
    } else {
        ext_fn_result = connect_to_queue(fn_id, fn_input);
    }
         */
    let ext_fn_result = socket_utils::connect_unix_socket(fn_id+fn_input).unwrap();
    let result = fn_id + fn_input + ext_fn_result;

    println!("Resume function A with result {} + {} + {} = {}",fn_id,fn_input,ext_fn_result,result);
    Ok(vec![WasmValue::from_i32(result)])
}

