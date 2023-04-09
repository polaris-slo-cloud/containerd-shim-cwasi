use std::ptr::null;
use log::info;
use oci_spec::runtime::Spec;
use walkdir::WalkDir;
use wasmedge_sdk::{Caller, WasmValue, host_function};
use wasmedge_sdk::error::HostFuncError;
use crate::{oci_utils, snapshot_utils, unix_socket};
use crate::unix_socket::ShimSocket;

pub static mut OCI_SPEC:Option<Spec> = None;
pub static mut BUNDLE_PATH:Option<String> = None;

#[host_function]
pub fn func_connect(_caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    let ext_fn_id = input[0].to_i32();
    let ext_fn_id_str = &ext_fn_id.to_string();
    let fn_input = input[1].to_i32();
    let mut ext_fn_result:i32 = 0;
    let external_fn_name: String;
    let socket_path: String;

    unsafe {
        //println!("host func specs {:?}", &spec);
        external_fn_name = oci_utils::get_wasm_annotations(&OCI_SPEC.clone().unwrap(), ext_fn_id_str);
        //get string until 2nd last / occurrence
        let bundle_path = BUNDLE_PATH.clone().unwrap().rsplitn(3, '/').nth(2).unwrap().to_string()+"/";
        socket_path= find_container_path(bundle_path.clone(), external_fn_name.clone());
    }
    println!("Connect to fn Id {} Name {}",ext_fn_id_str,external_fn_name);
    //check if the function is running locally
    let local_images_with_ext_fn_name = snapshot_utils::get_existing_image(vec![external_fn_name]);
    if local_images_with_ext_fn_name.is_empty(){
        println!("No local fn found. Connect to queue");
        //ext_fn_result = connect_to_queue(fn_id, fn_input);
    }else {
        println!("Connecting to {} with input {}",socket_path, fn_input);
        ext_fn_result = unix_socket::connect_unix_socket(fn_input, socket_path).unwrap();
    }

    let result = ext_fn_id + fn_input + ext_fn_result;

    println!("Resume function A with result {} + {} + {} = {}",ext_fn_id,fn_input,ext_fn_result,result);
    Ok(vec![WasmValue::from_i32(result)])
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
