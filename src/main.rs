use anyhow::Context;
use chrono::{DateTime, Utc};
use containerd_shim_wasm::sandbox::error::Error;
use containerd_shim_wasm::sandbox::{ShimCli};
use containerd_shim_wasm::sandbox::oci;
use containerd_shim_wasm::sandbox::{EngineGetter, Instance, InstanceConfig};
use libc::{dup, dup2, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use log::{error, info};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::os::unix::io::{IntoRawFd, RawFd};
use std::path::Path;
use std::sync::{
    mpsc::Sender,
    {Arc, Condvar, Mutex, RwLock},
};
use wasmedge_sys::AsyncResult;
use std::{process, thread};
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::sync::mpsc::channel;
use wasmedge_sdk::{ config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions}, ImportObjectBuilder, params, PluginManager, Vm};
use containerd_shim_cwasi::error::WasmRuntimeError;
use regex::Regex;
use containerd_shim_cwasi::{host_function_utils, oci_utils, snapshot_utils, shim_listener};
use itertools::Itertools;
use lazy_static::lazy_static;
use oci_spec::runtime::Spec;
use nix::{sys::signal, unistd::Pid};

static mut STDIN_FD: Option<RawFd> = None;
static mut STDOUT_FD: Option<RawFd> = None;
static mut STDERR_FD: Option<RawFd> = None;

lazy_static! {
    static ref JOB: Arc<RwLock<Option<AsyncResult>>> = Arc::new(RwLock::new(None));
}

type ExitCode = (Mutex<Option<(u32, DateTime<Utc>)>>, Condvar);
pub struct Wasi {
    exit_code: Arc<(Mutex<Option<(u32, DateTime<Utc>)>>, Condvar)>,
    engine: Vm,

    id: String,
    stdin: String,
    stdout: String,
    stderr: String,
    bundle: String,
}



pub fn reset_stdio() {
    unsafe {
        if STDIN_FD.is_some() {
            dup2(STDIN_FD.unwrap(), STDIN_FILENO);
        }
        if STDOUT_FD.is_some() {
            dup2(STDOUT_FD.unwrap(), STDOUT_FILENO);
        }
        if STDERR_FD.is_some() {
            dup2(STDERR_FD.unwrap(), STDERR_FILENO);
        }
    }
}

pub fn maybe_open_stdio(path: &str) -> Result<Option<RawFd>, Error> {
    if path.is_empty() {
        return Ok(None);
    }
    match OpenOptions::new().read(true).write(true).open(path) {
        Ok(f) => Ok(Some(f.into_raw_fd())),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Ok(None),
            _ => Err(err.into()),
        },
    }
}


pub fn prepare_module(mut vm: Vm, spec: &oci::Spec, stdin_path: String, stdout_path: String, stderr_path: String ) -> Result<Vm, WasmRuntimeError> {
    info!("opening rootfs");
    let rootfs_path = oci::get_root(spec).to_str().unwrap();
    let root = format!("/:{}", rootfs_path);
    let mut preopens = vec![root.as_str()];

    info!("opening mounts");
    let mut mounts = oci_utils::get_wasm_mounts(spec);
    preopens.append(&mut mounts);

    let args = oci::get_args(spec);
    info!("args {:?}", args);
    let envs = oci_utils::env_to_wasi(spec);
    info!("envs {:?}", envs);

    info!("opening stdin");
    let stdin = maybe_open_stdio(&stdin_path).context("could not open stdin")?;
    if stdin.is_some() {
        unsafe {
            STDIN_FD = Some(dup(STDIN_FILENO));
            dup2(stdin.unwrap(), STDIN_FILENO);
        }
    }

    info!("opening stdout");
    let stdout = maybe_open_stdio(&stdout_path).context("could not open stdout")?;
    if stdout.is_some() {
        unsafe {
            STDOUT_FD = Some(dup(STDOUT_FILENO));
            dup2(stdout.unwrap(), STDOUT_FILENO);
        }
    }

    info!("opening stderr");
    let stderr = maybe_open_stdio(&stderr_path).context("could not open stderr")?;
    if stderr.is_some() {
        unsafe {
            STDERR_FD = Some(dup(STDERR_FILENO));
            dup2(stderr.unwrap(), STDERR_FILENO);
        }
    }

    let mut cmd = args[0].clone();
    let stripped = args[0].strip_prefix(std::path::MAIN_SEPARATOR);
    if let Some(strpd) = stripped {
        cmd = strpd.to_string();
    }

    let mod_path = oci::get_root(spec).join(cmd);
    info!("register wasm app from file {:?}",mod_path);
    let mod_path_print = mod_path.clone();
    let additional_modules = extract_modules_from_wat(mod_path_print.as_path());
    for module_path in additional_modules {
        let additional_module = Path::new(module_path.as_str());
        let module_name = additional_module.file_name().unwrap().to_str().unwrap().replace(".wasm","");
        info!("additional module name {:?} {:?}",module_name, module_path);
        vm = vm.clone().register_module_from_file(module_name,additional_module)?;
    }

    info!("setting up wasi");
    let mut wasi_instance = vm.wasi_module()?;
    wasi_instance.initialize(
        Some(args.iter().map(|s| s as &str).collect()),
        Some(envs.iter().map(|s| s as &str).collect()),
        Some(preopens),
    );

    let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32), i32>("func_connect", host_function_utils::func_connect)?
        .build("cwasi_export")?;

    let vm= vm.register_import_module(import)?.register_module_from_file("main", mod_path)?;
    info!("module registered");
    Ok(vm)
}

pub fn extract_modules_from_wat(path: &Path) -> Vec<String>{
    let mod_wat = wasmprinter::print_file(path).unwrap();
    //info!("module wat {:?}",mod_wat);
    let re = Regex::new(r#"\bimport\s+\S+"#).unwrap();
    let matches = re.find_iter(mod_wat.as_str()).map(|s| s.as_str()).unique().collect_vec();
    let mut modules: Vec<String> = vec![];
    for cap in matches {
        let module = cap.replace("import ","").replace("\"","") + ".wasm";
        modules.push(module.to_string());
    }
    info!("extracted import modules from wat {:#?}", modules);
    let modules_path: Vec<String> = snapshot_utils::get_existing_image(modules);
    info!("Modules path: {:#?}",modules_path);
    return modules_path;
}


fn start_process(bundle_path: String, spec: Spec, vm: Vm) -> Result<(), Box<dyn std::error::Error>>{
    let secondary_function = oci_utils::get_wasm_annotations(&spec, "cwasi.secondary.function");
    println!("Secondary function {}",secondary_function);
    if secondary_function == "true" {
        //let _ret = match socket_utils::create_server_socket(vm,bundle_path){
        //let mut unix_socket = shim_listener::ShimListener::new(bundle_path.to_string(), spec, vm);
        match shim_listener::init_listener(bundle_path.to_string(), spec, vm) {
            Ok(_) => std::process::exit(0),
            Err(_) => std::process::exit(137),
        };

    }else {
        match vm.run_func(Some("main"), "_start", params!()) {
            Ok(_) => std::process::exit(0),
            Err(_) => std::process::exit(137),
        };
    }
}
impl Instance for Wasi {
    type E = Vm;
    fn new(_id: String, cfg: Option<&InstanceConfig<Self::E>>) -> Self {
        info!(">>> new instance");
        let cfg = cfg.unwrap();
        Wasi {
            exit_code: Arc::new((Mutex::new(None), Condvar::new())),
            engine: cfg.get_engine(),
            id,
            stdin: cfg.get_stdin().unwrap_or_default(),
            stdout: cfg.get_stdout().unwrap_or_default(),
            stderr: cfg.get_stderr().unwrap_or_default(),
            bundle: cfg.get_bundle().unwrap_or_default(),
        }
    }

    fn start(&self) -> Result<u32, Error> {
        info!(">>> call prehook before start");
        // Call prehook before the start
        let bundle = self.bundle.clone();
        let bundle_path = self.bundle.as_str();
        info!("bundle path {:?}", bundle_path);
        let spec = oci::load(Path::new(&bundle).join("config.json").to_str().unwrap())?;
        match spec.hooks().as_ref() {
            None => {
                log::debug!("no hooks found")
            }
            Some(hooks) => {
                let prestart_hooks = hooks.prestart().as_ref().unwrap();

                for hook in prestart_hooks {
                    let mut hook_command = process::Command::new(&hook.path());
                    // Based on OCI spec, the first argument of the args vector is the
                    // arg0, which can be different from the path.  For example, path
                    // may be "/usr/bin/true" and arg0 is set to "true". However, rust
                    // command differenciates arg0 from args, where rust command arg
                    // doesn't include arg0. So we have to make the split arg0 from the
                    // rest of args.
                    if let Some((arg0, args)) = hook.args().as_ref().and_then(|a| a.split_first()) {
                        log::debug!("run_hooks arg0: {:?}, args: {:?}", arg0, args);
                        hook_command.arg0(arg0).args(args)
                    } else {
                        hook_command.arg0(&hook.path().display().to_string())
                    };

                    let envs: HashMap<String, String> = if let Some(env) = hook.env() {
                        oci_utils::parse_env(env)
                    } else {
                        HashMap::new()
                    };
                    log::debug!("run_hooks envs: {:?}", envs);

                    let mut hook_process = hook_command
                        .env_clear()
                        .envs(envs)
                        .stdin(process::Stdio::piped())
                        .spawn()
                        .with_context(|| "Failed to execute hook")?;
                    let hook_process_pid = Pid::from_raw(hook_process.id() as i32);

                    if let Some(stdin) = &mut hook_process.stdin {
                        // We want to ignore BrokenPipe here. A BrokenPipe indicates
                        // either the hook is crashed/errored or it ran successfully.
                        // Either way, this is an indication that the hook command
                        // finished execution.  If the hook command was successful,
                        // which we will check later in this function, we should not
                        // fail this step here. We still want to check for all the other
                        // error, in the case that the hook command is waiting for us to
                        // write to stdin.
                        let state = format!("{{ \"pid\": {} }}", std::process::id());
                        if let Err(e) = stdin.write_all(state.as_bytes()) {
                            if e.kind() != ErrorKind::BrokenPipe {
                                // Not a broken pipe. The hook command may be waiting
                                // for us.
                                let _ = signal::kill(hook_process_pid, signal::Signal::SIGKILL);
                            }
                        }
                    }

                    hook_process.wait()?;
                }
            }
        }

        info!(">>> shim starts");
        let engine = self.engine.clone();

        let exit_code = self.exit_code.clone();
        let (tx, rx) = channel::<Result<(), Error>>();
        let stdin = self.stdin.clone();
        let stdout = self.stdout.clone();
        let stderr = self.stderr.clone();

        let _ = thread::Builder::new()
            .name(self.id.clone())
            .spawn(move || {
                info!("starting instance");

                info!("preparing module");
                let vm = match prepare_module(engine, &spec, stdin, stdout, stderr) {
                    Ok(vm) => vm,
                    Err(err) => {
                        tx.send(Err(Error::Others(err.to_string()))).unwrap();
                        return;
                    }
                };

                info!("notifying main thread we are about to start");
                tx.send(Ok(())).unwrap();

                info!("starting wasi instance");

                // TODO: How to get exit code?
                // This was relatively straight forward in go, but wasi and wasmtime are totally separate things in rust.
                let (lock, cvar) = &*exit_code;

                let mut job = JOB.write().unwrap();

                *job = Some(start_process(bundle,spec,vm).unwrap());

                    //vm.run_func_async(Some("main"), "_start", params!())
                    //    .unwrap(),

                drop(job);

                let _ret = match (&*JOB.read().unwrap()).as_ref().unwrap().get_async() {
                    Ok(_) => {
                        info!("exit code: {}", 0);
                        let mut ec = lock.lock().unwrap();
                        *ec = Some((0, Utc::now()));
                    }
                    Err(_) => {
                        error!("exit code: {}", 137);
                        let mut ec = lock.lock().unwrap();
                        *ec = Some((137, Utc::now()));
                    }
                };

                cvar.notify_all();
            })?;

        info!("Waiting for start notification");
        match rx.recv().unwrap() {
            Ok(_) => (),
            Err(err) => {
                info!("error starting instance: {:?}", err);
                let code = self.exit_code.clone();

                let (lock, cvar) = &*code;
                let mut ec = lock.lock().unwrap();
                *ec = Some((139, Utc::now()));
                cvar.notify_all();
                return Err(err);
            }
        }

        Ok(std::process::id())


        /*let spec = oci_utils::load_spec(self.bundle.clone())?;
        let bundle_path = self.bundle.as_str();
        info!("bundle path {:?}", bundle_path);
        info!("loading specs {:?}", spec);
        unsafe {
            crate::host_function_utils::OCI_SPEC=Some(spec.clone());
            crate::host_function_utils::BUNDLE_PATH=Some(bundle_path.to_string());
        }
        let vm = prepare_module(engine, &spec, stdin, stdout, stderr)
            .map_err(|e| Error::Others(format!("error setting up module: {:?}", e)))?;
        info!("vm created");
        let cg = oci::get_cgroup(&spec)?;

        oci::setup_cgroup(cg.as_ref(), &spec)
            .map_err(|e| Error::Others(format!("error setting up cgroups: {:?}", e)))?;
        let res = unsafe { exec::fork(Some(cg.as_ref())) }?;
        match res {
            exec::Context::Parent(tid, pidfd) => {
                let mut lr = self.pidfd.lock().unwrap();
                *lr = Some(pidfd.clone());

                info!("started wasi instance with tid {} at {}", tid,self.bundle.as_str());

                let code = self.exit_code.clone();
                let bundle_path = self.bundle.clone();
                let _ = thread::spawn(move || {
                    let (lock, cvar) = &*code;
                    let status = match pidfd.wait() {
                        Ok(status) => status,
                        Err(e) => {
                            error!("error waiting for pid {}: {:?}", tid, e);
                            oci_utils::delete(bundle_path).expect("static delete");
                            cvar.notify_all();
                            //pidfd.kill(SIGKILL as i32).expect("force kill");
                            return;
                        }
                    };

                    info!("wasi instance exited with status {}", status.status);
                    let mut ec = lock.lock().unwrap();
                    *ec = Some((status.status, Utc::now()));
                    drop(ec);
                    cvar.notify_all();
                });
                Ok(tid)
            }
            exec::Context::Child => {
                // child process

                // TODO: How to get exit code?
                // This was relatively straight forward in go, but wasi and wasmtime are totally separate things in rust.

                //TODO create queue with functionId
                let secondary_function = oci_utils::get_wasm_annotations(&spec, "cwasi.secondary.function");
                println!("Secondary function {}",secondary_function);
                if secondary_function == "true" {
                    //let _ret = match socket_utils::create_server_socket(vm,bundle_path){                    
                    //let mut unix_socket = shim_listener::ShimListener::new(bundle_path.to_string(), spec, vm);
                    let _ret = match shim_listener::init_listener(bundle_path.to_string(), spec, vm) {
                         Ok(_) => std::process::exit(0),
                        Err(_) => std::process::exit(137),
                    };

                }else {
                    let _ret = match vm.run_func(Some("main"), "_start", params!()) {
                        Ok(_) => std::process::exit(0),
                        Err(_) => std::process::exit(137),
                    };
                }
            }


        }

         */
    }

    fn kill(&self, signal: u32) -> Result<(), Error> {
        info!("killcw {}",self.bundle.as_str());
        let binding = self.bundle.as_str().to_owned() + ".sock";
        let socket_path = Path::new(&binding);
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).unwrap();
            info!("Socket {:?} deleted",self.bundle.as_str());
        }match &*JOB.read().unwrap() {
            Some(job) => {
                job.cancel();
                let code = self.exit_code.clone();
                let (lock, cvar) = &*code;
                let mut ec = lock.lock().unwrap();
                *ec = Some((137, Utc::now()));
                cvar.notify_one();
            }
            None => {
                // no running wasm task
            }
        };
        Ok(())

    }

    fn delete(&self) -> Result<(), Error> {
        info!("deletecw {}",self.bundle.as_str());
        let spec = match oci_utils::load_spec(self.bundle.clone()){
            Ok(spec) => spec,
            Err(err) => {
                error!("Could not load spec, skipping cgroup cleanup: {:?}", err);
                return Ok(());
            }
        };
        let cg = oci::get_cgroup(&spec)?;
        cg.delete()?;

        let binding = self.bundle.as_str().to_owned() + ".sock";
        let socket_path = Path::new(&binding);
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).unwrap();
            info!("Socket {:?} deleted",self.bundle.as_str());
        }
        Ok(())
    }

    fn wait(&self, channel: Sender<(u32, DateTime<Utc>)>) -> Result<(), Error> {
        info!("wait");
        let code = self.exit_code.clone();
        thread::spawn(move || {
            let (lock, cvar) = &*code;
            let mut exit = lock.lock().unwrap();
            while (*exit).is_none() {
                exit = cvar.wait(exit).unwrap();
            }
            let ec = (*exit).unwrap();
            match channel.send(ec) {
                Ok(_) => {}
                Err(error) => {
                    error!("channel disconnect: {}", error);
                }
            }
        });

        Ok(())
    }
}

impl EngineGetter for Wasi {
    type E = Vm;
    fn new_engine() -> Result<Vm, Error> {
        info!("new engine");
        PluginManager::load_from_default_paths();
        let mut host_options = HostRegistrationConfigOptions::default();
        host_options = host_options.wasi(true);
        #[cfg(all(target_os = "linux", feature = "wasi_nn", target_arch = "x86_64"))]
        {
            host_options = host_options.wasi_nn(true);
        }
        let config = ConfigBuilder::new(CommonConfigOptions::default())
            .with_host_registration_config(host_options)
            .build()
            .map_err(anyhow::Error::msg)?;
        let vm = Vm::new(Some(config)).map_err(anyhow::Error::msg)?;
        Ok(vm)
    }
}


fn main() {
    containerd_shim::run::<ShimCli<Wasi, wasmedge_sdk::Vm>>("io.containerd.cwasi.v1", None);
}
