use serde::{Serialize,Deserialize};

#[derive(Debug,Serialize,Deserialize)]
pub struct Message{
    pub channel: String,
    pub message_id: String,
    pub payload: i32
}

impl Message {
    pub fn new(message_id: String, channel: String,  payload: i32) -> Message {
        Message {
            channel,
            message_id,
            payload
        }
    }
}
