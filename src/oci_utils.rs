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

pub fn get_wasm_mounts(spec: &Spec,annotation_key: &str) -> Vec<&str> {
    let annotations: Vec<&str> = match spec.mounts() {
        Some(annotations) => annotations
            .iter()
            .filter_map(|annotation| {

                if let Some(typ) = annotation.typ() {
                    if annotation == "bind" || typ == "tmpfs" {
                        return annotation.destination().to_str();
                    }
                }

                None
            })
            .collect(),
        _ => vec![],
    };
    annotations
}

pub fn get_wasm_mounts(spec: &Spec) -> Vec<&str> {

}
