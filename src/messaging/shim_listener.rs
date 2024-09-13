use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use anyhow::Error;
use oci_spec::runtime::Spec;
use wasmedge_sdk::{params, Vm};
use crate::messaging::message::Message;
use chrono;
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use crate::messaging::redis_utils;
use crate::utils::oci_utils;
use crate::utils::time_utils::epoch_todate;

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
                                                              message.source_channel, message.payload,message.start));
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
        loop {
            match listener.accept() {
                Ok((mut socket, _addr)) => {
                    // Read data from the socket stream
                    let mut reader = BufReader::new(socket.try_clone()?);
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(_) => {
                            if line != "exit"{
                                let client_input = line.trim();
                                reader.into_inner();
                                let result = self.call_vm_with_input(client_input).unwrap();
                                //TODO add the string
                                let client_response = format!("hello world from from fnB socket server. Result from Module B : {}", result);
                                socket.write_all(client_response.as_bytes())?;
                                socket.flush()?;
                            }
                        }
                        Err(err) => eprintln!("Error reading line: {}", err),
                    }
                },
                Err(e) => println!("accept function failed: {:?}", e),
            }
        }
    }

    fn call_vm_with_input(&mut self, input: &str) -> Result<i64, Box<dyn std::error::Error>>{

        let num2: i32 = 15;
        let args = oci_utils::arg_to_wasi(&self.oci_spec);
        //println!("setting up wasi at at : {}", chrono::offset::Utc::now());
        let my_strings: [&str; 3] = [&args.first().unwrap(), input, &num2.to_string()];
        let my_vector: Vec<&str> = my_strings.to_vec();
        //println!("args: {}", my_vector.join(", "));

        // Set new arguments on the wasi instance
        let vm = self.vm.as_mut().unwrap();
        let mut wasi_instance = vm.wasi_module()?;
        wasi_instance.initialize(
            Some(my_vector),
            Some(vec![]),
            Some(vec![]),
        );
        //let start = chrono::offset::Utc::now();
        //println!("Run wasm func at {:?}",chrono::offset::Utc::now());
        let res = vm.run_func(Some("main"), "cwasi_function", params!())?;
        //let end= chrono::offset::Utc::now();
        //println!("Run func finished at {:?} Duration {}",end,end-start);
        let result = res[0].to_i64();
        let end_date = epoch_todate(result);
        let message: Message = serde_json::from_str(&input).unwrap();
        println!("FnB Shim Finished. Result from moduleB: {}",result);
        let start_date: DateTime<Utc> = message.start.parse::<DateTime<Utc>>().unwrap();
        //println!("EndDate {} and StartDate: {:?}",end_date,start_date);
        // Calculate the duration between the two dates
        //let _: Duration = end_date.signed_duration_since(start_date);
        //println!("Duration in milliseconds: {}", duration.num_milliseconds());


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
    stream.write_all(input_fn_b.as_bytes()).unwrap();

    let mut response = String::new();
    println!("start reading response {}",Utc::now());
    stream.read_to_string(&mut response)?;
    Ok(response)

}

#[tokio::main(flavor = "current_thread")]
pub async fn init_listener(bundle_path: String, oci_spec: Spec, vm: Vm) -> Result<(), Box<dyn std::error::Error>>{
    println!("before init");
    let mut listener = ShimListener::new(bundle_path.clone(), oci_spec.clone(), vm.clone());
    let input = String::from_utf8_lossy(&*connect_to_source("127.0.0.1:8080".to_string())?).to_string();
    listener.call_vm_with_input(input.as_str()).expect("TODO: panic message");
    /*let channel = oci_utils::arg_to_wasi(&oci_spec).first().unwrap().replace("/","").replace(".wasm","");
    match listener.subscribe(&channel).await {
        Ok(_result) => {
            println!("channel created {}",channel);
        }
        Err(_) => std::process::exit(137),
    }
    let mut listener2 = ShimListener::new(bundle_path, oci_spec.clone(), vm);
    listener2.create_server_socket()?;
     */
    Ok(())
}

fn connect_to_source(address: String) -> Result<Vec<u8>, Box<dyn std::error::Error>>{
    // Connect to the provided IP address and port
    let mut stream = TcpStream::connect(address.clone())?;
    println!("Connected to the server at {}", address);

    let mut buffer = Vec::new();

    // Read all the data from the server until the connection is closed
    let bytes_read = stream.read_to_end(&mut buffer)?;

    let end_time = chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    println!("Received {} bytes at {:?}", bytes_read, end_time);

    Ok(buffer)
}