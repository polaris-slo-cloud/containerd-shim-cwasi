use std::collections::hash_map::Entry;
use oci_spec::runtime::Spec;


pub fn env_to_wasi(spec: &Spec) -> Vec<String> {
    let default = vec![];
    let env = spec
        .process()
        .as_ref()
        .unwrap()
        .env()
        .as_ref()
        .unwrap_or(&default);
    env.to_vec()
}

pub fn get_wasm_mounts(spec: &Spec) -> Vec<&str> {
    let mounts: Vec<&str> = match spec.mounts() {
        Some(mounts) => mounts
            .iter()
            .filter_map(|mount| {
                if let Some(typ) = mount.typ() {
                    if typ == "bind" || typ == "tmpfs" {
                        return mount.destination().to_str();
                    }
                }
                None
            })
            .collect(),
        _ => vec![],
    };
    mounts
}

pub fn get_wasm_annotations(spec: &Spec,annotation_key: &str) -> String {
    let annotations = &spec.annotations();
    let map = &spec.annotations().as_ref().unwrap();
    let myentry = map.get(annotation_key);
    let value: String = myentry.map_or_else(String::default, |s| s.to_owned());
    return value;
}


