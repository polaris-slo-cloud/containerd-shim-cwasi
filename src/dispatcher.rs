use std::path::Path;
use std::thread;
use std::time::Duration;
use log::info;
use oci_spec::runtime::Spec;
use uuid::Uuid;
use walkdir::WalkDir;
use wasmedge_sdk::{Caller, WasmValue, host_function};
use wasmedge_sdk::error::HostFuncError;
use crate::{oci_utils, redis_utils, shim_listener};
use crate::message::Message;

pub static mut OCI_SPEC:Option<Spec> = None;
pub static mut BUNDLE_PATH:Option<String> = None;

#[host_function]
pub fn func_connect(caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    println!("Host function invoked at {}",chrono::offset::Utc::now());
    thread::sleep(Duration::from_secs(2));
    let mut mem = caller.memory(0).unwrap();
    let arg1_ptr = input[0].to_i32() as u32;
    let arg1_len = input[1].to_i32() as u32;
    println!("External function input length {}",arg1_len);
    let mut external_function_type = mem.read_string(arg1_ptr, arg1_len).expect("fail to get string");
    let message_obj: Message = serde_json::from_str(&external_function_type).unwrap();
    external_function_type = message_obj.target_channel;
    println!("Function target {}",external_function_type);

    let socket_path: String;
    let ext_func_result:String;
    unsafe {
        let bundle_path = BUNDLE_PATH.clone().unwrap().rsplitn(3, '/').nth(2).unwrap().to_string()+"/";
        socket_path= find_container_path(bundle_path.clone(), external_function_type.clone());
    }
    if socket_path.is_empty() {
        ext_func_result = connect_to_queue(external_function_type.replace(".wasm",""), message_obj.payload);
    }else {
        ext_func_result = shim_listener::connect_unix_socket(message_obj.payload, socket_path).unwrap();
        //UNTIL HERE
    }
    let bytes = ext_func_result.as_bytes();
    let len = bytes.len();
    mem.write(bytes, arg1_ptr).unwrap();
    Ok(vec![WasmValue::from_i32(len as i32)])

}




fn connect_to_queue(channel :String, fn_target_input:String) -> String{

    let fn_source_id = Uuid::new_v4().to_simple().to_string();
    let fn_source_id_copy = fn_source_id.clone();
    let _ = redis_utils::publish_message(Message::new(fn_source_id,
                                                      channel, fn_target_input)).unwrap();
    let result = redis_utils::_subscribe(fn_source_id_copy.as_str());
    return result.payload;
}


fn find_container_path(path:String, function_name:String) -> String {
    for file in WalkDir::new(path).into_iter().filter_map(|file| file.ok()) {
        let file_name = file.file_name().to_str().unwrap();
        if file.metadata().unwrap().is_file() && file_name=="config.json" {
            info!("oci config spec found: {}", file.path().display());
            let c_path = file.path().display().to_string().replace("/config.json","");
            let spec = oci_utils::load_spec(c_path.clone()).unwrap();
            let args = oci_utils::arg_to_wasi(&spec);
            let c_path_formatted=args.first().unwrap().to_string().replace("/","");
            if c_path_formatted==function_name && Path::new(&(c_path.clone()+".sock")).exists(){
                return c_path;
            }
        }
    }
    return String::new();
}
