use std::path::Path;
use log::info;
use oci_spec::runtime::Spec;
use uuid::Uuid;
use walkdir::WalkDir;
use wasmedge_sdk::{Caller, WasmValue, host_function};
use wasmedge_sdk::error::HostFuncError;
use crate::{oci_utils, redis_utils, shim_listener};
use crate::message::Message;
use chrono::{DateTime, Utc};
use regex::Regex;

pub static mut OCI_SPEC:Option<Spec> = None;
pub static mut BUNDLE_PATH:Option<String> = None;

#[host_function]
pub fn func_connect(caller: Caller, input: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    println!("Host function invoked at {}",chrono::offset::Utc::now());
    let mut mem = caller.memory(0).unwrap();
    let arg1_ptr = input[0].to_i32() as u32;
    let arg1_len = input[1].to_i32() as u32;
    println!("External function input length {}",arg1_len);
    let mut external_function_type = mem.read_string(arg1_ptr, arg1_len).expect("fail to get string");
    let message_obj: Message = serde_json::from_str(&external_function_type).unwrap();
    //println!("message obj {:?}",message_obj);
    external_function_type = message_obj.target_channel;

    /*let arg2_ptr = input[0].to_i32() as u32;
    let arg2_len = input[1].to_i32() as u32;
    let payload = mem.read_string(arg2_ptr, arg2_len).expect("fail to get string");

     */
    //println!("Payload {}",payload);

    let socket_path: String;
    let mut ext_func_result:String;

    unsafe {
        //external_fn_name = oci_utils::get_wasm_annotations(&OCI_SPEC.clone().unwrap(), ext_fn_id_str);
        //get string until 2nd last / occurrence
        let bundle_path = BUNDLE_PATH.clone().unwrap().rsplitn(3, '/').nth(2).unwrap().to_string()+"/";
        socket_path= find_container_path(bundle_path.clone(), external_function_type.clone());
    }
    //check if the function is running locally
    //let local_images_with_ext_fn_name = snapshot_utils::get_existing_image(vec![external_fn_name]);

    //println!("External func {:?}",chrono::offset::Utc::now());
    //ext_func_result = connect_to_queue(external_function_type.replace(".wasm",""), payload);
    let start: DateTime<Utc> = chrono::offset::Utc::now();
    if socket_path.is_empty() {
        ext_func_result = connect_to_queue(external_function_type.replace(".wasm",""), message_obj.payload);
    }else {
        println!("Connecting to {:?} at {:?}",socket_path, start);
        ext_func_result = shim_listener::connect_unix_socket(message_obj.payload, socket_path).unwrap();
        //THIS IS JUST FOR THE FAN-IN FAN-OUT
        let re = Regex::new(r"Z(.*)").unwrap();
        let data_result = re.replace(&ext_func_result.replace("Received from client at ",""),"").to_string();
        println!("Date result {}",data_result);
        let datetime = DateTime::parse_from_rfc3339(&*format!("{}{}",data_result,"Z"))
            .unwrap_or_else(|err| panic!("Failed to parse date string: {}", err));

        // Convert the DateTime to the Utc timezone
        let datetime_utc: DateTime<Utc> = datetime.into();

        // Extract the date
        let duration_b = datetime_utc - start ;
        ext_func_result = duration_b.num_microseconds().unwrap().to_string();
        //UNTIL HERE
    }


    let bytes = ext_func_result.as_bytes();
    let len = bytes.len();
    mem.write(bytes, arg1_ptr);


   // println!("Resume function with result from ext func  {}",len);
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
