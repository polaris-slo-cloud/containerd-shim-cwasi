extern crate redis;


use std::error::Error;
use chrono::{DateTime, SecondsFormat};
use log::info;
use redis::{Client, Commands, Connection, ControlFlow, PubSubCommands, RedisResult};
use crate::message::Message;
use lazy_static::lazy_static;
use std::sync::Mutex;


struct RedisConnection {
    client: Client,
}

lazy_static! {
    static ref REDIS_IP:String = std::env::var("REDIS_IP").unwrap_or("192.168.0.38".to_string());
    static ref REDIS_CONNECTION: Mutex<RedisConnection> = {
        let client = Client::open("redis://".to_owned()+&REDIS_IP).unwrap();
        let connection = RedisConnection { client };
        Mutex::new(connection)
    };
}


pub fn connect() -> redis::Connection {
    let redis_connection = REDIS_CONNECTION.lock().unwrap();
    return redis_connection.client.get_connection().unwrap();

}


pub fn publish_message(message: Message) -> Result<String, Box<dyn Error>> {
    //println!("Get connection: {}",chrono::offset::Utc::now());
    let mut con = connect();
    let json = serde_json::to_string(&message).unwrap();
    let payload = json.as_str();
    let start= chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    println!("Connecting to queue {:?} at {:?}",message.target_channel,start );
    con.publish(message.target_channel.clone(), payload)?;
    println!("After publish at {:?}",chrono::offset::Utc::now());
    //println!("Published message to channel: {}",message.target_channel);
    Ok(start.to_string())
}


pub fn _subscribe(channel: &str) -> Message {
    println!("before Subscribe to channel: {} {}", channel,chrono::offset::Utc::now());
    let mut connection = connect();
    //println!("redis connection created");
    let mut pubsub = connection.as_pubsub();
    pubsub.subscribe(channel).unwrap();
    println!("After Subscribed to channel: {} at {}", channel,chrono::offset::Utc::now());
    // set timeouts in seconds
    //pubsub.set_read_timeout(Some(std::time::Duration::new(60, 0))).unwrap();
    let msg = pubsub.get_message().unwrap();
    let start= chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    println!("Got messsage channel {} at  {}",channel, chrono::offset::Utc::now());

    let res_time=format!("Received from client at : {}", start);

    let payload: String = msg.get_payload().unwrap();
    //THIS IS ONLY FOR THE FANOUT TEST

    //UNTIL HERE
    let mut message_obj: Message = serde_json::from_str(&payload).unwrap();
    //println!("Received message source: {} target: {}", message_obj.source_channel,message_obj.target_channel);
    //overwrite this for exp measurement
    if channel=="func_b"{
        message_obj.payload=res_time;
    }
    println!("returning message_obj {}  {}",channel, message_obj.payload);
    return message_obj;
}