use anyhow::{Context, Result};
use containerd_shim_wasm::container::{Engine, Entrypoint, Instance, RuntimeContext, Stdio};
use wasmedge_sdk::config::{ConfigBuilder, HostRegistrationConfigOptions};
use wasmedge_sdk::plugin::PluginManager;
use crate::sandbox::cwasi_vm::{CwasiVm};

pub type WasmEdgeInstance = Instance<WasmEdgeEngine>;
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
        let spec = ctx.specs();
        let args = ctx.args();
        let envs: Vec<_> = std::env::vars().map(|(k, v)| format!("{k}={v}")).collect();
        let Entrypoint {
            source,
            func,
            arg0: _,
            name,
        } = ctx.entrypoint();

        //Source::Oci([module]) => Ok(Cow::Borrowed(&module.layer))
        //log::info!("source {source:?}");
        log::info!("spec {spec:?}");
        log::info!("name {name:?}");
        log::info!("arg {args:?}");
        log::info!("env {envs:?}");

        let mut vm = self.vm.clone();
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