use log::info;
use walkdir::WalkDir;

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