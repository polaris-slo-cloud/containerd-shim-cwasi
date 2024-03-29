extern crate redis;


use std::error::Error;
use redis::Commands;
use crate::message::Message;

fn connect() -> redis::Connection {
    //format - host:port
    /*let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
     */

    let redis_conn_url = format!("redis://127.0.0.1");
    //println!("{}", redis_conn_url);

    redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

pub fn read(key: &str) -> String {
    let mut conn = connect();
    let key_str= key.clone();

    let result = match conn.get(key_str) {
        Ok(val) => val,
        Ok(None) => Some(String::new()),
        Err(e) => panic!("Error getting value: {}", e),
    };
    let value = result.unwrap_or("".to_owned());

    println!("KVS value for '{}' = {}", key_str,value);
    return value;
}

pub fn set(key: &str, value: &str) {
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
    let json = serde_json::to_string(&message).unwrap();
    let payload = json.as_str();

    con.publish(message.target_channel, payload)?;
    println!("Published message: {}",payload);
    Ok(())
}


pub fn subscribe(channel: &str) -> Message {
    let mut connection = connect();
    let mut pubsub = connection.as_pubsub();

    pubsub.subscribe(channel).unwrap();
    println!("Subscribed to channel: {}", channel);
    // set timeouts in seconds
    //pubsub.set_read_timeout(Some(std::time::Duration::new(60, 0))).unwrap();

    let msg = pubsub.get_message().unwrap();
    let payload: String = msg.get_payload().unwrap();
    println!("Received message: {}", payload);

    let message_obj: Message = serde_json::from_str(&payload).unwrap();

    return message_obj;
}