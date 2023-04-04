use serde::{Serialize,Deserialize};


#[derive(Debug,Serialize,Deserialize)]
pub struct Message{
    pub source_channel: String,
    pub target_channel: String,
    pub payload: i32
}

impl Message {
    pub fn new(source_channel: String, target_channel: String,  payload: i32) -> Message {
        Message {
            source_channel,
            target_channel,
            payload
        }
    }
}


