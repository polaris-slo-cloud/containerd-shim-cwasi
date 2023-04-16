use log::info;
use oci_spec::runtime::Spec;
use uuid::Uuid;
use walkdir::WalkDir;
use wasmedge_sdk::{Caller, WasmValue, host_function, params};
use wasmedge_sdk::error::HostFuncError;
use crate::{oci_utils, redis_utils, shim_listener};
use crate::message::Message;

pub static mut OCI_SPEC:Option<Spec> = None;
pub static mut BUNDLE_PATH:Option<String> = None;

#[host_function]
pub fn func_connect(caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    println!("Host function");
    let mut mem = caller.memory(0).unwrap();
    let arg1_ptr = input[0].to_i32() as u32;
    let arg1_len = input[1].to_i32() as u32;
    let external_function_type = mem.read_string(arg1_ptr, arg1_len).expect("fail to get string");
    println!("External function type {}",external_function_type);

    let arg2_ptr = input[0].to_i32() as u32;
    let arg2_len = input[1].to_i32() as u32;
    let payload = mem.read_string(arg2_ptr, arg2_len).expect("fail to get string");
    println!("Payload {}",payload);

    let socket_path: String;
    let ext_func_result:String;

    unsafe {
        //external_fn_name = oci_utils::get_wasm_annotations(&OCI_SPEC.clone().unwrap(), ext_fn_id_str);
        //get string until 2nd last / occurrence
        let bundle_path = BUNDLE_PATH.clone().unwrap().rsplitn(3, '/').nth(2).unwrap().to_string()+"/";
        socket_path= find_container_path(bundle_path.clone(), external_function_type.clone());
    }
    //check if the function is running locally
    //let local_images_with_ext_fn_name = snapshot_utils::get_existing_image(vec![external_fn_name]);


    if socket_path.is_empty() {
        println!("No local fn found. Connect to queue");
        ext_func_result = connect_to_queue(external_function_type.replace(".wasm",""), payload);
    }else {
        println!("Connecting to {} with input {}",socket_path, payload);
        ext_func_result = shim_listener::connect_unix_socket(payload, socket_path).unwrap();
    }

    let input = String::from("this is a string create to be written on the memory");
    let bytes = input.as_bytes();
    let len = bytes.len();
    mem.write(bytes, arg1_ptr);


    println!("Resume function with result from ext func  {}",ext_func_result);
    Ok(vec![WasmValue::from_i32(len as i32)])
}


fn connect_to_queue(channel :String, fn_target_input:String) -> String{
    println!("Connecting to queue {} with input {}",channel, fn_target_input);
    let fn_source_id = Uuid::new_v4().to_simple().to_string();
    let fn_source_id_copy = fn_source_id.clone();

    let _ = redis_utils::publish_message(Message::new(fn_source_id,
                                                      channel, fn_target_input));
    let result = redis_utils::_subscribe(fn_source_id_copy.as_str());
    return result;
}


fn find_container_path(path:String, function_name:String) -> String {
    for file in WalkDir::new(path).into_iter().filter_map(|file| file.ok()) {
        let file_name = file.file_name().to_str().unwrap();
        if file.metadata().unwrap().is_file() && file_name=="config.json" {
            info!("oci config spec found: {}", file.path().display());
            let c_path = file.path().display().to_string().replace("/config.json","");
            let spec = oci_utils::load_spec(c_path.clone()).unwrap();
            let args = oci_utils::arg_to_wasi(&spec);
            if args.first().unwrap().to_string().replace("/","")==function_name{
                return c_path;
            }
        }
    }
    return String::new();
}
