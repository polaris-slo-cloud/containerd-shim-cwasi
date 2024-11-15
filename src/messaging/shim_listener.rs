use std::ffi::CString;

use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use anyhow::Error;
use oci_spec::runtime::Spec;
use wasmedge_sdk::{params, AsInstance, Vm, WasmVal};
use crate::messaging::message::Message;
use chrono;
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use crate::messaging::redis_utils;
use crate::utils::{oci_utils, snapshot_utils};
use std::result::Result;
use crate::utils::time_utils::epoch_todate;
use std::fs::File;
use std::{io, mem, ptr, slice};
use std::os::unix::io::FromRawFd;
use libc::{c_int, pipe, size_t, splice, SPLICE_F_MORE, SPLICE_F_MOVE};
use crate::messaging::dispatcher::BUNDLE_PATH;
extern crate libc;

use std::sync::atomic::{AtomicBool, Ordering};

#[repr(C)] // Ensure the structure has a C-compatible layout
pub struct SharedData {
    size: u32, // Size of the valid data
    data: [u8; 501 * 1024 * 1024],
    is_ready: AtomicBool,
}

#[derive(Clone)]
pub struct ShimListener {
    pub bundle_path: String,
    pub oci_spec: Spec,
    pub vm: Option<Vm>
}

impl ShimListener {
    pub fn new(bundle_path: String, oci_spec: Spec, wasm_vm: Vm) -> ShimListener {
        ShimListener {
            bundle_path,
            oci_spec,
            vm :Some(wasm_vm)
        }
    }

    pub async fn subscribe(&mut self, channel:&str) -> Result<std::thread::JoinHandle<()>, redis::RedisError>{
        let owned_queue_name = channel.to_owned();
        let mut listener = self.clone();
        Ok(std::thread::spawn(move || loop {
            let mut con = redis_utils::connect();
            let mut pubsub = con.as_pubsub();

            pubsub.subscribe(format!("{}", owned_queue_name)).unwrap();
            let payload: String = pubsub.get_message().unwrap().get_payload().unwrap();

            let message: Message = serde_json::from_str(&payload).unwrap();

            println!("returning message_obj from to {}  {}",owned_queue_name, message.source_channel);
            let _ =listener.call_vm_with_input(message.payload.as_bytes().to_vec()).unwrap();
            let _=redis_utils::publish_message(Message::new(owned_queue_name.to_string(),
                                                              message.source_channel, message.payload,message.start));
            listener.stop_socket().unwrap();
            break;

        }))
    }

    unsafe fn handle_connection(&mut self, mut socket: UnixStream) -> Result<(), Box<dyn std::error::Error>> {

        let mut chunk = [0u8; 4];


        // Pre-allocate a buffer for chunked reading (8 KB in this case)
        let mut buffer = Vec::new(); // Buffer to accumulate the entire input

        let mut reader = BufReader::new(socket.try_clone()?);
        loop {
            let bytes_read = reader.read(&mut chunk)?;
            if bytes_read == 0 {
                // Client closed the connection
                break;
            }
            buffer.extend_from_slice(&chunk[..bytes_read]);
        }


        //println!("Received {} bytes at {}", buffer.len(), Utc::now());
       // println!("finished socket at {:?}", Utc::now());
        if !buffer.is_empty() {
            let result = self.call_vm_with_input(buffer)?;
            let client_response = format!(
                "hello world from fnB socket server. Result from Module B: {}",
                result
            );
            socket.write_all(client_response.as_bytes())?;
            socket.flush()?;
        }
        Ok(())
    }

    pub fn read_from_memory(&mut self){
        let sh_name = oci_utils::arg_to_wasi(&self.oci_spec).first().unwrap().replace("/","").replace(".wasm","");
        let mut shm_fd;
        println!("sh_name {}",sh_name);
        // Open the existing shared memory segment
        loop {
            shm_fd = unsafe {
                libc::shm_open(
                    CString::new(format!("/{}", sh_name)).unwrap().as_ptr(),
                    libc::O_RDONLY,
                    0,
                )
            };

            if shm_fd >= 0 {
                //println!("Shared memory opened successfully.");
                break;
            } else {
                std::thread::sleep(std::time::Duration::from_micros(500)); // Wait and retry
            }
        }


        // Map the shared memory segment
        let shared_data_ptr: *mut SharedData = unsafe {
            libc::mmap(ptr::null_mut(), mem::size_of::<SharedData>(), libc::PROT_READ, libc::MAP_SHARED, shm_fd, 0) as *mut SharedData
        };


        if shared_data_ptr.is_null() {
            eprintln!("{}", io::Error::last_os_error());
        }

        // Read data from shared memory
        let shared_data = unsafe { &*shared_data_ptr }; // Create an immutable reference to the shared data

        let size = shared_data.size as usize; // Get the size of the valid data
        let vector = &shared_data.data[..size];
        //println!("Read {} from shared memory at: {}", size, Utc::now());
        let _ = self.call_vm_with_input(vector.to_vec()).unwrap();
        //let message = String::from_utf8_lossy(vector);
        //println!("After serial from shared memory at: {}", Utc::now());// Convert the byte slice to a string
        //println!("Message {}", message);
        // Clean up


        unsafe {
            libc::munmap(shared_data_ptr as *mut libc::c_void, mem::size_of::<SharedData>());
            libc::close(shm_fd);
            libc::shm_unlink(CString::new(sh_name).unwrap().as_ptr());
        }
    }

    pub fn create_server_socket(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        let binding = self.bundle_path.to_owned() + ".sock";
        let socket_path = Path::new(&binding);
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).unwrap();
        }

        let listener = UnixListener::bind(&socket_path)?;
        println!("Socket created successfully at {:?} {}", &socket_path, Utc::now());
        for stream in listener.incoming().next() {
            match stream {
                Ok(socket) => unsafe {
                    // Handle the connection (consider using threads or async for concurrency)
                    self.handle_connection(socket).unwrap_or_else(|e| eprintln!("Error: {}", e));
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
        Ok(())
    }

    fn call_vm_with_input(&mut self, input: Vec<u8>) -> Result<i64, Box<dyn std::error::Error>>{
        //println!("Value from func a {}",input);
        //let num2: i32 = 15;
        //let args = oci_utils::arg_to_wasi(&self.oci_spec);
        //println!("setting up wasi at at : {}", chrono::offset::Utc::now());
        //let my_strings: [&str; 3] = [&args.first().unwrap(), input, &num2.to_string()];
        //let my_vector: Vec<&str> = my_strings.to_vec();
        //println!("args: {}", my_vector.join(", "));

        // Set new arguments on the wasi instance
        let vm = self.vm.as_mut().unwrap();
        //let mut wasi_instance = vm.wasi_module()?;
        /*wasi_instance.initialize(
            Some(vec![]),
            Some(vec![]),
            Some(vec![]),
        );

         */
        //let start = chrono::offset::Utc::now();
        //println!("Run wasm func at {:?}",chrono::offset::Utc::now());
        let main_instance = vm.named_module("main").unwrap();

        let allocate = main_instance.func("allocate").unwrap();
        let len = input.len() as i32;
        let result = allocate.call(vm, params!(len)).unwrap();
        let func_addr = result[0].to_i32();
        let mut memory = main_instance.memory("memory").unwrap();
        let _ = memory.write(input, func_addr as u32);

        let cwasi_func = main_instance.func("hello_greet").unwrap();
        let res = cwasi_func.call(vm, params!(func_addr, len)).unwrap();
        //let res = vm.run_func(Some("main"), "cwasi_function", params!())?;

        //let end= chrono::offset::Utc::now();
        //println!("Run func finished at {:?} Duration {}",end,end-start);
       // let result = res[0].to_i64();
        //let end_date = epoch_todate(result);
        //let message: Message = serde_json::from_str(&input).unwrap();
        println!("FnB Shim Finished. Result from moduleB: {}",5);
        //let start_date: DateTime<Utc> = message.start.parse::<DateTime<Utc>>().unwrap();
        //println!("EndDate {} and StartDate: {:?}",end_date,start_date);
        // Calculate the duration between the two dates
        //let _: Duration = end_date.signed_duration_since(start_date);
        //println!("Duration in milliseconds: {}", duration.num_milliseconds());


        Ok(5)
    }
    pub fn stop_socket (&self) -> Result<(), Box<dyn std::error::Error>>{
        let binding = self.bundle_path.as_str().to_owned() + ".sock";
        connect_unix_socket(String::from("exit").into_bytes(),self.bundle_path.as_str().to_owned())?;
        let socket_path = Path::new(&binding);
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).unwrap();
            println!("Socket {:?} deleted",self.bundle_path.as_str());
        }
        Ok(())
    }
}

//socket_path = find_container_path_parallel(BUNDLE_PATH.as_deref().unwrap_or(""), external_function_type)
pub fn connect_unix_socket(input_fn_a:Vec<u8>, mut socket_path: String) -> Result<String, Error> {
    //connect to socket
    //println!("Connecting to {:?}", socket_path);

    const MAX_RETRIES: u32 = 1000; // Maximum value for u32 (4,294,967,295)

    let mut retries = 0;
    let mut stream: UnixStream;

    loop {
        match UnixStream::connect(socket_path.clone() + ".sock") {
            Ok(s) => {
                stream = s;
                break;
            },
            Err(err) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    panic!("Exceeded maximum retries, failed to connect to socket.");
                }
                socket_path = unsafe{snapshot_utils::find_container_path_parallel(BUNDLE_PATH.as_deref().unwrap_or(""), "alice-lib.wasm")};
            }
        }
    }

    //let mut stream = UnixStream::connect(socket_path+".sock").unwrap();
    if let Err(e) = stream.write_all(input_fn_a.as_slice()) {
        eprintln!("Failed to write data: {:?}", e);
    }
    stream.shutdown(std::net::Shutdown::Write).expect("shutdown failed");
    let mut response = String::new();
    //println!("start reading response {}",Utc::now());
    stream.read_to_string(&mut response)?;
    Ok(response)
}

pub unsafe fn send_shared_memory(message:Vec<u8>,) -> Result<String, Error> {

    // Create shared memory
    let shim_name ="/func_b".to_string();
    let shm_fd = unsafe { libc::shm_open(CString::new(shim_name).unwrap().as_ptr(), libc::O_CREAT | libc::O_RDWR, 0o600) };

    if shm_fd < 0 {
        eprintln!("{}", io::Error::last_os_error());
    }

    // Set the size of the shared memory segment
    if unsafe { libc::ftruncate(shm_fd, size_of::<SharedData>() as i64) } < 0 {
        eprintln!("{}", io::Error::last_os_error());
    }

    // Map the shared memory segment
    let shared_data_ptr: *mut SharedData = unsafe {
        libc::mmap(ptr::null_mut(), size_of::<SharedData>(), libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, shm_fd, 0) as *mut SharedData
    };

    if shared_data_ptr.is_null() {
        eprintln!("{}", io::Error::last_os_error());
    }

    //println!("Write {} shared memory at: {}",message.len(), Utc::now());
    // Write data to shared memory
    let shared_data = unsafe { &mut *shared_data_ptr }; // Create a mutable reference to the shared data
    shared_data.size = message.len() as u32;
    shared_data.data[..shared_data.size as usize].copy_from_slice(message.as_slice()); // Copy the message into the data field

    //shared_data.is_ready.store(true, Ordering::SeqCst);
    //println!("Written to shared memory: {}", String::from_utf8_unchecked(message.clone()));

    // Clean up
    unsafe {
        libc::munmap(shared_data_ptr as *mut libc::c_void, mem::size_of::<SharedData>());
        libc::close(shm_fd);
    }
    Ok("finished".to_string())
}


#[tokio::main(flavor = "current_thread")]
pub async fn init_listener(bundle_path: String, oci_spec: Spec, vm: Vm) -> Result<(), Box<dyn std::error::Error>>{
    println!("before init");
    let mut listener = ShimListener::new(bundle_path.clone(), oci_spec.clone(), vm.clone());
    let input = connect_to_source("128.131.57.188:8080".to_string())?;
    listener.call_vm_with_input(input).expect("TODO: panic message");
    /*let channel = oci_utils::arg_to_wasi(&oci_spec).first().unwrap().replace("/","").replace(".wasm","");
    match listener.subscribe(&channel).await {
        Ok(_result) => {
            println!("channel created {}",channel);
        }
        Err(_) => std::process::exit(137),
    }

     */
    //let mut listener = ShimListener::new(bundle_path, oci_spec.clone(), vm);
    //listener.create_server_socket().expect("socket creation error");
    //listener2.read_from_memory();
    Ok(())
}

fn connect_to_source(address: String) -> Result<Vec<u8>, Box<dyn std::error::Error>>{

    let mut buffer = Vec::new();
    loop {
        match TcpStream::connect(address.clone()) {
            Ok(mut stream) => {
                //println!("Connected to the server at {}", address);
                // Read all the data from the server until the connection is closed
                let bytes_read = stream.read_to_end(&mut buffer)?;

                let end_time = chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
                //println!("Received {} bytes at {:?}", bytes_read, end_time);
                return Ok(buffer);
            }
            Err(e) => {
                std::thread::sleep(std::time::Duration::from_micros(500)); // Wait and retry
            }
        }
    }

    // Connect to the provided IP address and port
    let mut stream = TcpStream::connect(address.clone())?;
    println!("Connected to the server at {}", address);

}