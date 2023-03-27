extern crate redis;

use std::error::Error;
use redis::Commands;
use crate::message::Message;

fn connect() -> redis::Connection {
    //format - host:port
    /*let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
     */

    let redis_conn_url = format!("http://127.0.0.1");
    //println!("{}", redis_conn_url);

    redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

pub fn read(key: String) -> String {
    let mut conn = connect();

    let value: String = redis::cmd("GET")
        .arg(key)
        .query(&mut conn)
        .expect(format!("failed to execute GET for '{}'", key).as_str());
    println!("value for '{}' = {}", key,value);
    return value;
}

pub fn set(key: String, value: String) {
    let mut conn = connect();

    let _: () = redis::cmd("SET")
        .arg(key)
        .arg(value)
        .query(&mut conn)
        .expect(format!("failed to execute SET for '{}'",key).as_str());
    println!("value stored for '{}' = {}", key,value);
}

pub fn publish_message(message: Message) -> Result<(), Box<dyn Error>> {
    let mut con = connect();
    let json = serde_json::to_string(&message);

    con.publish(message.channel, json)?;
    println!("Published message: {}",message.payload);
    Ok(())
}


pub fn subscribe(channel: String) -> i32 {
    let mut pubsub = connect().as_pubsub();
    pubsub.subscribe(channel).unwrap();
    println!("Subscribed to channel: {}", channel);
    let msg = pubsub.get_message().unwrap();
    let payload: String = msg.get_payload().unwrap();
    println!("Received message: {}", payload);
    //let message_obj = serde_json::from_str::<Message>(&payload).unwrap();
    let message_obj: Message = serde_json::from_str(&payload).unwrap();
    return message_obj.payload;
}