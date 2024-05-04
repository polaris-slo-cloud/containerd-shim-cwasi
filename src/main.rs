use std::env;
use std::fs::File;
use containerd_shim_wasm::sandbox::cli::{revision, shim_main, version};
use containerd_shim_cwasi::sandbox::engine::WasmEdgeInstance;

fn main() {
    File::create("log").expect("Error creating log file");
    shim_main::<WasmEdgeInstance>("cwasi", version!(), revision!(), "v1", None);
}