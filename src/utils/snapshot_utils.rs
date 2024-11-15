use std::path::Path;
use log::info;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;
use crate::utils::oci_utils;
use reqwest::blocking::Client;

const CONTAINERD_SNAPSHOT: &'static str ="/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs/snapshots";

pub fn get_existing_image(image_names: Vec<String>) -> Vec<String>{
    let mut images_path: Vec<String> = vec![];
    for file in WalkDir::new(CONTAINERD_SNAPSHOT).into_iter().filter_map(|file| file.ok()) {
        let file_name = file.file_name().to_str().unwrap();
        if file.metadata().unwrap().is_file() && image_names.contains(&file_name.to_string()){
            images_path.push(file.path().display().to_string());
            info!("image path found: {}", file.path().display());
        }
    }
    return images_path;
}


pub fn find_container_path_parallel(path: &str, function_name: &str) -> String {
    let paths: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|file| file.ok())
        .collect();

    paths.par_iter().find_map_any(|file| {
        let metadata = file.metadata().ok()?;
        if metadata.is_file() {
            let file_name = file.file_name().to_str()?;
            if file_name == "config.json" {
                let c_path = file.path().display().to_string().replace("/config.json", "");
                let spec = oci_utils::load_spec(c_path.clone()).ok()?;
                let args = oci_utils::arg_to_wasi(&spec);
                let c_path_formatted = args.first()?.to_string().replace("/", "");
                if c_path_formatted == function_name && Path::new(&(c_path.clone() + ".sock")).exists() {
                    return Some(c_path);
                }
            }
        }
        None
    }).unwrap_or_else(|| String::new())
}


pub fn download_file_content() -> String {
    let url = format!("http://127.0.0.1:8888/files/file_10M.txt");
    println!("downloading {}", url);
    let response = reqwest::blocking::get(url).unwrap().text().unwrap();
    response
}