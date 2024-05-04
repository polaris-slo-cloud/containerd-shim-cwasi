use nix::errno::Errno;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{env, thread};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use nix::sys::wait::{waitid, Id as WaitID, WaitPidFlag, WaitStatus};
use anyhow::Context;
use chrono::{DateTime, Utc};
use containerd_shim_wasm::container::{Engine, Instance};
use containerd_shim_wasm::sandbox::instance_utils::{determine_rootdir, get_instance_root, instance_exists};
use containerd_shim_wasm::sandbox::sync::WaitableCell;
use containerd_shim_wasm::sys::container::executor::Executor;
use containerd_shim_wasm::sandbox::{containerd, Error as SandboxError, Error, Instance as SandboxInstance, InstanceConfig, Stdio};
use libc::mode_t;
use oci_spec::image::Platform;
use libcontainer::container::builder::ContainerBuilder;
use libcontainer::container::Container;
use libcontainer::signal::Signal;
use libcontainer::syscall::syscall::SyscallType;
use libcontainer::tty::StdIO::Stdin;
use nix::unistd::Pid;
use wasmedge_sdk::Vm;
use crate::sandbox::cwasi_vm::CwasiVm;
use crate::sandbox::engine::WasmEdgeEngine;


static DEFAULT_CONTAINER_ROOT_DIR: &str = "/run/container";

pub struct CwasiInstance<E: Engine> {
    rootdir: PathBuf,
    id: String,
    exit_code: WaitableCell<(u32, DateTime<Utc>)>,
    _phantom: PhantomData<E>,
    bundle: PathBuf,
    stdio: Stdio
}

impl<E: Engine + std::fmt::Debug> SandboxInstance for CwasiInstance<E>
{
    type Engine = E;


    fn new(id: String, cfg: Option<&InstanceConfig<Self::Engine>>) -> Result<Self, SandboxError> {
        let cfg = cfg.context("missing configuration")?;
        let engine = cfg.get_engine();
        log::info!("cwasi engine: {:?}", engine.clone());

        let bundle = cfg.get_bundle().to_path_buf();
        log::info!("cwasi creating new instance bundle: {:?}", bundle.clone().as_path());
        let namespace = cfg.get_namespace();
        log::info!("cwasi namespace: {}", namespace);
        let rootdir = Path::new(DEFAULT_CONTAINER_ROOT_DIR).join(E::name());
        let rootdir = determine_rootdir(&bundle, &namespace, rootdir)?;
        log::info!("cwasi rootdir: {:?}", rootdir.as_path());
        let stdio = Stdio::init_from_cfg(cfg)?;

        match env::current_dir() {
            Ok(path) => log::info!("Current directory is: {}", path.display()),
            Err(e) => log::info!("Error getting current directory: {}", e),
        }

        // check if container is OCI image with wasm layers and attempt to read the module
        let (modules, platform) = containerd::Client::connect(cfg.get_containerd_address().as_str(), &namespace)?
            .load_modules(&id, &engine)
            .unwrap_or_else(|e| {
                log::warn!("Error obtaining wasm layers for container {id}.  Will attempt to use files inside container image. Error: {e}");
                (vec![], Platform::default())
            });

        log::info!("cwasi modules: {:?} platform {:?}", modules, platform);
        ContainerBuilder::new(id.clone(), SyscallType::Linux)
            .with_executor(Executor::new(engine, stdio.clone(), modules, platform))
            .with_root_path(rootdir.clone())?
            .as_init(&bundle)
            .with_systemd(false)
            .build()?;

        Ok(Self {
            id,
            exit_code: WaitableCell::new(),
            rootdir,
            _phantom: Default::default(),
            bundle,
            stdio
        })
    }

    /// Start the instance
    /// The returned value should be a unique ID (such as a PID) for the instance.
    /// Nothing internally should be using this ID, but it is returned to container where a user may want to use it.
    fn start(&self) -> Result<u32, SandboxError> {
        log::info!("cwasi starting instance: {}", self.id);
        // make sure we have an exit code by the time we finish (even if there's a panic)
        let guard = self.exit_code.set_guard_with(|| (137, Utc::now()));

        let container_root = get_instance_root(&self.rootdir, &self.id)?;
        log::info!("cwasi container root: {:?}", container_root.as_path());
        //let mut vm = self.wasm_engine.get_vm().to_owned();
        //vm.prepare_module(self.stdio.clone())
        //    .map_err(|e| Error::Others(format!("error setting up module: {}", e)))?;
        let mut container = Container::load(container_root)?;
        let pid = container.pid().context("failed to get pid")?.as_raw();

        container.start()?;


        let exit_code = self.exit_code.clone();
        thread::spawn(move || {
            // move the exit code guard into this thread
            let _guard = guard;

            let status = match waitid(WaitID::Pid(Pid::from_raw(pid)), WaitPidFlag::WEXITED) {
                Ok(WaitStatus::Exited(_, status)) => status,
                Ok(WaitStatus::Signaled(_, sig, _)) => sig as i32,
                Ok(_) => 0,
                Err(Errno::ECHILD) => {
                    log::info!("no child process");
                    0
                }
                Err(e) => {
                    log::error!("waitpid failed: {e}");
                    137
                }
            } as u32;
            let _ = exit_code.set((status, Utc::now()));
        });

        Ok(pid as u32)
    }

    /// Send a signal to the instance
    fn kill(&self, signal: u32) -> Result<(), SandboxError> {
        log::info!("sending signal {signal} to instance: {}", self.id);
        let signal = Signal::try_from(signal as i32).map_err(|err| {
            SandboxError::InvalidArgument(format!("invalid signal number: {}", err))
        })?;
        let container_root = get_instance_root(&self.rootdir, &self.id)?;
        let mut container = Container::load(container_root)
            .with_context(|| format!("could not load state for container {}", self.id))?;

        container.kill(signal, true)?;

        Ok(())
    }

    /// Delete any reference to the instance
    /// This is called after the instance has exited.
    fn delete(&self) -> Result<(), SandboxError> {
        log::info!("deleting instance: {}", self.id);
        match instance_exists(&self.rootdir, &self.id) {
            Ok(true) => {}
            Ok(false) => return Ok(()),
            Err(err) => {
                log::error!("could not find the container, skipping cleanup: {}", err);
                return Ok(());
            }
        }
        let container_root = get_instance_root(&self.rootdir, &self.id)?;
        match Container::load(container_root) {
            Ok(mut container) => {
                container.delete(true)?;
            }
            Err(err) => {
                log::error!("could not find the container, skipping cleanup: {}", err);
            }
        }
        Ok(())
    }

    /// Waits for the instance to finish and retunrs its exit code
    /// Returns None if the timeout is reached before the instance has finished.
    /// This is a blocking call.
    fn wait_timeout(&self, t: impl Into<Option<Duration>>) -> Option<(u32, DateTime<Utc>)> {
        self.exit_code.wait_timeout(t).copied()
    }
}