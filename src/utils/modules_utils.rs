use std::fs::File;
use std::io::Read;
use std::path::Path;
use itertools::Itertools;
use log::info;
use regex::Regex;


use serde::{Deserialize, Serialize};
use crate::utils::snapshot_utils;

#[derive(Serialize, Deserialize, Debug)]
struct Bundle {
    bundle: String
}

static DEFAULT_CONTAINER_ROOT_DIR: &str = "/run/container/wasmedge/";
static DEFAULT_STATE_FILE: &str = "state.json";
static DEFAULT_ROOTFS_DIR: &str = "rootfs/";

pub fn extract_modules_from_wat(path: &Path) -> Vec<String>{
    let mod_wat = wasmprinter::print_file(path).unwrap();
    //info!("module wat {:?}",mod_wat);
    let re = Regex::new(r#"\bimport\s+\S+"#).unwrap();
    let matches = re.find_iter(mod_wat.as_str()).map(|s| s.as_str()).unique().collect_vec();
    let mut modules: Vec<String> = vec![];
    for cap in matches {
        let module = cap.replace("import ","").replace("\"","") + ".wasm";
        modules.push(module.to_string());
    }
    info!("extracted import modules from wat {:#?}", modules);
    let modules_path: Vec<String> = snapshot_utils::get_existing_image(modules);
    info!("Modules path: {:#?}",modules_path);
    return modules_path;
}


pub fn get_bundle_path(f_name: &str) -> String {
    let binding = Path::new(DEFAULT_CONTAINER_ROOT_DIR).join(f_name).join(DEFAULT_STATE_FILE);
    let path = binding.as_path() ;
    info!("path {:#?}", path);
    let mut file = File::open(path).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Deserialize the JSON data into the Config struct
    let config: Bundle = serde_json::from_str(&contents).unwrap();

    info!("bundle path {:#?}", config);
    return config.bundle;

}