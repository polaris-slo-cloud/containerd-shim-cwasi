use anyhow::{ Result};
use log::info;
use wasmedge_sdk::error::WasmEdgeError;
use wasmedge_sdk::{VmBuilder, WasmEdgeResult, WasmValue};
use wasmedge_sdk::config::Config;
use wasmedge_sdk::wasi::WasiInstance;
use crate::utils::modules_utils::get_bundle_path;

#[derive(Debug)]
#[derive(Clone)]
pub struct CwasiVm {
    vm: wasmedge_sdk::Vm,
    pub config: Config,
}impl CwasiVm {
    pub fn new(config: Config) -> Result<Self, WasmEdgeError> {
        log::info!("config {config:?}");
        let vm = VmBuilder::new().with_config(config.clone()).build().unwrap();
        Ok(Self {
            vm,
            config
        })
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

        info!("named instances {:?}",self.vm.instance_names());
        let name = mod_name.as_ref();
        info!("register wasm app from file {:?}",name);
        /*let bundle =  get_bundle_path(name);

        let mod_path_print = mod_path.clone();
        let additional_modules = extract_modules_from_wat(mod_path_print.as_path());
        for module_path in additional_modules {
            let additional_module = Path::new(module_path.as_str());
            let module_name = additional_module.file_name().unwrap().to_str().unwrap().replace(".wasm","");
            info!("additional module name {:?} {:?}",module_name, module_path);
            //vm = vm.clone().register_module_from_file(module_name,additional_module)?;
        }*/

        self.vm = self.vm.register_module_from_bytes(&mod_name, wasm_bytes).unwrap();
        Ok(self)
    }

    pub fn wasi_module_mut(&mut self) -> Option<&mut WasiInstance> {
        return self.vm.wasi_module_mut();
    }

    pub fn wasi_module(&self) -> Option<&WasiInstance> {
        return self.vm.wasi_module();
    }


}

