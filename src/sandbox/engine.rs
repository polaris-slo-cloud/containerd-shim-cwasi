use std::borrow::Cow;
use anyhow::{Context, Result};
use containerd_shim_wasm::container::{Engine, Entrypoint, Instance, RuntimeContext, Source, Stdio};
use wasmedge_sdk::config::{ConfigBuilder, HostRegistrationConfigOptions};
use wasmedge_sdk::plugin::PluginManager;
use wasmedge_sdk::Vm;
use crate::sandbox::cwasi_instance::CwasiInstance;
use crate::sandbox::cwasi_vm::{CwasiVm};

pub type WasmEdgeInstance = CwasiInstance<WasmEdgeEngine>;
#[derive(Debug)]
#[derive(Clone)]
pub struct WasmEdgeEngine {
    vm: CwasiVm,
}
impl Default for WasmEdgeEngine {
    fn default() -> Self {
        let host_options = HostRegistrationConfigOptions::default();
        let host_options = host_options.wasi(true);
        let config = ConfigBuilder::default()
            .with_host_registration_config(host_options)
            .build()
            .unwrap();
        let vm= CwasiVm::new(config).unwrap();
        Self { vm }
    }
}

impl Engine for WasmEdgeEngine {
    fn name() -> &'static str {
        "wasmedge"
    }
    fn run_wasi(&self, ctx: &impl RuntimeContext, stdio: Stdio) -> Result<i32> {
        log::info!("run_wasi start");
        let args = ctx.args();
        log::info!("arg {args:?}");
        let envs: Vec<_> = std::env::vars().map(|(k, v)| format!("{k}={v}")).collect();
        log::info!("env {envs:?}");
        let Entrypoint {
            source,
            func,
            arg0: _,
            name,
        } = ctx.entrypoint();
        log::info!("name {name:?}");
        let mut vm = self.vm.clone();
        vm.set_vm_properties(ctx.specs());
        log::info!("vm {vm:?}");
        log::info!("source {source:?}");

        //let mut vm = self.vm.clone();
        vm.wasi_module_mut()
            .context("Not found wasi module")?
            .initialize(
                Some(args.iter().map(String::as_str).collect()),
                Some(envs.iter().map(String::as_str).collect()),
                Some(vec!["/:/"]),
            );

        let mod_name = name.unwrap_or_else(|| "main".to_string());
        log::info!("modname {mod_name:?}");
        PluginManager::load(None)?;
        let vm = vm.auto_detect_plugins()?;

        let wasm_bytes = source.as_bytes()?;
        let vm = vm
            .register_module_from_bytes(&mod_name,  wasm_bytes)
            .context("registering module")?;

        stdio.redirect()?;

        log::info!("running with method {func:?}");
        vm.run_func(Some(&mod_name), func, vec![])?;
        log::info!("env {envs:?}");
        let status = vm
            .wasi_module()
            .context("Not found wasi module")?
            .exit_code();

        Ok(status as i32)
    }
}