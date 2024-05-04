use std::env;
use std::path::{Path, PathBuf};
use anyhow::{ Result};
use containerd_shim_wasm::sandbox::{oci, Stdio};
use log::info;
use oci_spec::runtime::Spec;
use regex::Regex;
use wasmedge_sdk::error::WasmEdgeError;
use wasmedge_sdk::{ImportObjectBuilder, Vm, VmBuilder, WasmEdgeResult, WasmValue};
use wasmedge_sdk::config::Config;
use wasmedge_sdk::wasi::WasiInstance;
use crate::error::WasmRuntimeError;
use crate::messaging::dispatcher;
use crate::utils::modules_utils::extract_modules_from_wat;
use crate::utils::oci_utils;

#[derive(Clone,Debug)]
pub struct CwasiVm {
    vm: Vm,
    specs: Option<Spec>,
    bundle: Option<PathBuf>,
    container_name: Option<String>,
    root_dir: Option<PathBuf>,
}impl CwasiVm {
    pub fn new(config: Config) -> Result<Self, WasmEdgeError> {
        log::info!("config {config:?}");
        let vm = VmBuilder::new().with_config(config.clone()).build().unwrap();
        Ok(Self {
            vm,
            specs: None,
            bundle: None,
            container_name: None,
            root_dir: None
        })
    }

    pub fn set_vm_properties(&mut self, specs: Spec){
        let specs_clone = Some(specs.clone());
        let re = Regex::new(r"([^/]+)/[^/]+$").expect("Invalid regex pattern");
        self.specs = specs_clone;
        if let Some(root) = specs.root() {
            log::info!("path {:?}", root.path());
            self.root_dir = Some(root.path().to_path_buf());
            log::info!("root dir {:?}",self.root_dir);
            self.bundle = Some(root.path().parent().unwrap().to_path_buf());
            log::info!("bundle {:?}",self.bundle);
            self.container_name = Some(re.captures(root.path().as_os_str().to_str().unwrap()).unwrap().get(1).unwrap().as_str().parse().unwrap())
        }
    }
    pub fn run_func(
        &self,
        mod_name: Option<&str>,
        func_name: impl AsRef<str>,
        args: impl IntoIterator<Item = WasmValue>,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        self.vm.run_func(mod_name, func_name, args)
    }

    pub fn auto_detect_plugins(mut self) -> WasmEdgeResult<Self> {
        self.vm = self.vm.auto_detect_plugins()?;
        Ok(self)
    }

    pub fn register_module_from_bytes(
        mut self,
        mod_name: impl AsRef<str>,
        wasm_bytes: impl AsRef<[u8]>,
    ) -> WasmEdgeResult<Self> {

        match env::current_dir() {
            Ok(path) => log::info!("Current directory is: {}", path.display()),
            Err(e) => log::info!("Error getting current directory: {}", e),
        }
        info!("named instances {:?}",self.vm.instance_names());
        let name = mod_name.as_ref();
        let root = self.root_dir.clone().unwrap();
        let mod_path = root.join(name.to_owned() +".wasm");
        info!("register wasm app from file {:?}",mod_path);
        //let mod_path_print = mod_path.clone();
        let additional_modules = extract_modules_from_wat(mod_path.as_path());
        for module_path in additional_modules {
            let additional_module = Path::new(module_path.as_str());
            let module_name = additional_module.file_name().unwrap().to_str().unwrap().replace(".wasm","");
            info!("additional module name {:?} {:?}",module_name, module_path);
            //vm = vm.clone().register_module_from_file(module_name,additional_module)?;
        }


        self.vm = self.vm.register_module_from_bytes(&mod_name, wasm_bytes).unwrap();
        Ok(self)
    }

    pub fn wasi_module_mut(&mut self) -> Option<&mut WasiInstance> {
        return self.vm.wasi_module_mut();
    }

    pub fn wasi_module(&self) -> Option<&WasiInstance> {
        return self.vm.wasi_module();
    }

    pub fn prepare_module(&mut self, stdio: Stdio) -> std::result::Result<Vm, WasmRuntimeError> {
        info!("opening rootfs");

        let root_clone = self.root_dir.clone();
        let root = root_clone.map_or_else(|| String::new(), |p| p.display().to_string());
        let mut preopens = vec![root.as_str()];

        info!("opening mounts");
        let mut mounts = oci_utils::get_wasm_mounts(self.specs.clone().unwrap());
        let mut mounts_vec = mounts.iter().map(|s| s.as_str()).collect();
        preopens.append(&mut mounts_vec);

        let args =oci_utils::arg_to_wasi(&self.specs.clone().unwrap());

        info!("args {:?}", args);
        let envs = oci_utils::env_to_wasi(&self.specs.clone().unwrap());
        info!("envs {:?}", envs);

        let mut cmd = args[0].clone();
        let stripped = args[0].strip_prefix(std::path::MAIN_SEPARATOR);
        if let Some(strpd) = stripped {
            cmd = strpd.to_string();
        }
        let name = args.get(0);
        let root = self.root_dir.clone().unwrap();
        let mod_path = root.join(name.unwrap().as_str().to_owned() +".wasm");
        info!("register wasm app from file {:?}",mod_path);
        let mod_path_print = mod_path.clone();
        let additional_modules = extract_modules_from_wat(mod_path_print.as_path());
        for module_path in additional_modules {
            let additional_module = Path::new(module_path.as_str());
            let module_name = additional_module.file_name().unwrap().to_str().unwrap().replace(".wasm","");
            info!("additional module name {:?} {:?}",module_name, module_path);
            self.vm = self.vm.clone().register_module_from_file(module_name,additional_module)?;
        }

        info!("setting up wasi");
        let mut wasi_instance = self.vm.wasi_module().unwrap();
        wasi_instance.to_owned().initialize(
            Some(args.iter().map(|s| s as &str).collect()),
            Some(envs.iter().map(|s| s as &str).collect()),
            Some(preopens),
        );

        //TODO: fix method
        //let import = ImportObjectBuilder::new()
        //    .with_func::<(i32, i32), i32, i32>("func_connect", dispatcher::func_connect, None)?
        //   .build("cwasi_export", None)?;

        //let vm= self.vm.register_import_module(import.borrow())?.register_module_from_file("main", mod_path)?;
        info!("module registered");
        let vm = self.vm.clone();
        Ok(vm)
    }


}

