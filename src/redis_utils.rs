extern crate redis;


use std::error::Error;
use chrono::SecondsFormat;
use log::info;
use redis::{Commands, ControlFlow, PubSubCommands, RedisResult};
use crate::message::Message;

pub fn connect() -> redis::Connection {
    let redis_ip = std::env::var("REDIS_IP").unwrap_or("192.168.0.207".to_string());
    println!("Value of REDIS_IP: {}", redis_ip);

    let client =match redis::Client::open("redis://".to_owned()+&redis_ip){
        Ok(client) => client,
        Err(err) => {
            eprintln!("Error subscribing to channel: {}", err);
            std::process::exit(1);
        }
    };
    return client.get_connection().unwrap();
}

pub fn publish_message(message: Message) -> Result<(), Box<dyn Error>> {
    let mut con = connect();
    let json = serde_json::to_string(&message).unwrap();
    let payload = json.as_str();

    con.publish(message.target_channel.clone(), payload)?;
    println!("Published message to channel: {}",message.target_channel);
    Ok(())
}

pub fn publish_string(message: String) -> Result<(), Box<dyn Error>> {
    let mut con = connect();

    con.publish(message.clone(), "My test message")?;
    println!("Published message: {}",message);
    Ok(())
}

/*pub fn subscribe(channel: &str) -> Result<(), Box<dyn Error>> {
    let _ = tokio::spawn(async move {
        let mut connection = connect();

        let _: () = connection.subscribe(&[channel], |msg| {
            let received: String = msg.get_payload().unwrap();
            let message_obj = serde_json::from_str::<Message>(&received).unwrap();

            message_handler::handler(message_obj);

            return ControlFlow::Continue;
        }).unwrap();
    });

    Ok(())
}
 */

pub fn _subscribe(channel: &str) -> Message {
    println!("Subscribe to channel: {}", channel);
    let mut connection = connect();
    println!("redis connection created");
    let mut pubsub = connection.as_pubsub();
    pubsub.subscribe(channel).unwrap();
    println!("Subscribed to channel: {}", channel);
    // set timeouts in seconds
    //pubsub.set_read_timeout(Some(std::time::Duration::new(60, 0))).unwrap();

    let msg = pubsub.get_message().unwrap();
    let payload: String = msg.get_payload().unwrap();
    //THIS IS ONLY FOR THE FANOUT TEST
    let start= chrono::offset::Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
    let res_time=format!("Received from client at : {}", start);
    //UNTIL HERE
    let mut message_obj: Message = serde_json::from_str(&payload).unwrap();
    println!("Received message source: {} target: {}", message_obj.source_channel,message_obj.target_channel);
    message_obj.payload=res_time;

    return message_obj;
}