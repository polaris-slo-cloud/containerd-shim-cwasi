use std::fs::File;
use std::io::Read;
use std::path::Path;
use itertools::Itertools;
use log::{error, info};
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
    info!("path {:?}",path);
    //let mod_wat = wasmprinter::print_file(path).unwrap();
    let mod_wat = wasmprinter::print_file(path).unwrap_or_else(|e| {
        error!("Error printing WAT from file: {:?}", e);
        String::from("")
    });
    info!("module wat {:?}",mod_wat);
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
