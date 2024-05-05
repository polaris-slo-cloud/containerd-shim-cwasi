use std::error::Error;
use redis::{Client, Commands};
use crate::messaging::message::Message;
use lazy_static::lazy_static;
use std::sync::Mutex;


struct RedisConnection {
    client: Client,
}

lazy_static! {
    static ref REDIS_IP:String = std::env::var("REDIS_IP").unwrap_or("192.168.0.207".to_string());
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


pub fn publish_message(message: Message) -> Result<(), Box<dyn Error>> {
    let mut con = connect();
    let json = serde_json::to_string(&message).unwrap();
    let payload = json.as_bytes();
    let channel = message.target_channel.clone();
    println!("Connecting to queue {:?}",message.target_channel);
    con.publish(channel, payload)?;
    println!("After publish at {:?}",chrono::offset::Utc::now());
    Ok(())
}


pub fn _subscribe(channel: &str) -> Message {
    println!("before Subscribe to channel: {} {}", channel,chrono::offset::Utc::now());
    let mut connection = connect();
    let mut pubsub = connection.as_pubsub();
    pubsub.subscribe(channel).unwrap();
    println!("After Subscribed to channel: {} at {}", channel,chrono::offset::Utc::now());
    let msg = pubsub.get_message().unwrap();
    println!("Got messsage channel {} at  {}",channel, chrono::offset::Utc::now());

    let payload: String = msg.get_payload().unwrap();
    let message_obj: Message = serde_json::from_str(&payload).unwrap();

    println!("returning message_obj {}  {}",channel, message_obj.payload);
    return message_obj;
}