use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use anyhow::Error;
use oci_spec::runtime::Spec;
use wasmedge_sdk::{params, Vm};
use crate::{oci_utils, redis_utils};
use crate::message::Message;
use chrono;
use chrono::{SecondsFormat};



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
            let _ =listener.call_vm_with_input(&message.payload).unwrap();
            let _=redis_utils::publish_message(Message::new(owned_queue_name.to_string(),
                                                              message.source_channel, message.payload));
            listener.stop_socket().unwrap();
            break;

        }))
    }

    pub fn create_server_socket(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        let binding = self.bundle_path.to_owned() + ".sock";
        let socket_path = Path::new(&binding);
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).unwrap();
        }

        let listener = UnixListener::bind(&socket_path)?;
        println!("Socket created successfully at {:?} {}", &socket_path, chrono::offset::Utc::now());
        match listener.accept() {
            Ok((mut socket, _addr)) => {
                // Read data from the socket stream
                let mut reader = BufReader::new(socket.try_clone()?);
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(_) => {
                        if line != "exit"{
                            let start= chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
                            let client_input = line.trim();
                            let res_time=format!("Received from client at {} : lenght {}", start,client_input.len());
                            // Send a response back to the client
                            reader.into_inner();
                            // Call function Code here
                            let result = self.call_vm_with_input(client_input).unwrap();

                            socket.write_all(res_time.as_bytes())?;
                        }
                    }
                    Err(err) => eprintln!("Error reading line: {}", err),
                }
            },
            Err(e) => println!("accept function failed: {:?}", e),
        }
        Ok(())
    }

    fn call_vm_with_input(&mut self, input: &str) -> Result<i32, Box<dyn std::error::Error>>{

        let num2: i32 = 15;
        let args = oci_utils::arg_to_wasi(&self.oci_spec);
        println!("setting up wasi at at : {}", chrono::offset::Utc::now());
        let my_strings: [&str; 3] = [&args.first().unwrap(), input, &num2.to_string()];
        let my_vector: Vec<&str> = my_strings.to_vec();

        // Set new arguments on the wasi instance
        let vm = self.vm.as_mut().unwrap();
        let mut wasi_instance = vm.wasi_module()?;
        wasi_instance.initialize(
            Some(my_vector),
            Some(vec![]),
            Some(vec![]),
        );
        let start = chrono::offset::Utc::now();
        println!("Run wasm func at {:?}",chrono::offset::Utc::now());
        let res = vm.run_func(Some("main"), "cwasi_function", params!())?;
        let end= chrono::offset::Utc::now();
        println!("Run func finished at {:?} Duration {}",end,end-start);
        let result = res[0].to_i32();
        println!("FnB Shim Finished. Result from moduleB: {}",result);

        Ok(result)
    }
    pub fn stop_socket (&self) -> Result<(), Box<dyn std::error::Error>>{
        let binding = self.bundle_path.as_str().to_owned() + ".sock";
        connect_unix_socket(String::from("exit"),self.bundle_path.as_str().to_owned())?;
        let socket_path = Path::new(&binding);
        if socket_path.exists() {
            std::fs::remove_file(&socket_path).unwrap();
            println!("Socket {:?} deleted",self.bundle_path.as_str());
        }
        Ok(())
    }
}


pub fn connect_unix_socket(input_fn_a:String, socket_path: String)-> Result<String, Error> {
    //connect to socket
    let mut stream = UnixStream::connect(socket_path+".sock").unwrap();
    let input_fn_b = format!("Data input from source fn {} \n", input_fn_a);
    //write request in the socket
    stream.write_all(input_fn_b.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    println!("{}",response);
    Ok(response.replace("Received from client at ", "").replace("\n",""))

}

#[tokio::main(flavor = "current_thread")]
pub async fn init_listener(bundle_path: String, oci_spec: Spec, vm: Vm) -> Result<(), Box<dyn std::error::Error>>{
    println!("before init");
    let mut listener = ShimListener::new(bundle_path.clone(), oci_spec.clone(), vm.clone());
    let channel = oci_utils::arg_to_wasi(&oci_spec).first().unwrap().replace("/","").replace(".wasm","");
    match listener.subscribe(&channel).await {
        Ok(_result) => {
            println!("channel created {}",channel);
        }
        Err(_) => std::process::exit(137),
    }

    let mut listener2 = ShimListener::new(bundle_path, oci_spec.clone(), vm);
    listener2.create_server_socket()?;
    println!("finished init listener");
    Ok(())
}