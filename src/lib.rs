pub mod oci_utils;
pub mod error;
pub mod host_function_utils;
pub mod shim_listener;
pub mod snapshot_utils;
pub mod message;
pub mod redis_utils;



pub fn handle(body: Vec<u8>) -> Result<Vec<u8>, Error> {
    let file_to_download = &String::from_utf8(body).unwrap();
    let response=download(file_to_download.to_string());
    Ok(PHRASE.as_bytes().to_vec())
}