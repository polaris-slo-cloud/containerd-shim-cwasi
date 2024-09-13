use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};


#[derive(Debug,Serialize,Deserialize)]
pub struct Message{
    pub source_channel: String,
    pub target_channel: String,
    pub payload: String,
    pub start: String,
}

impl Message {
    pub fn new(source_channel: String, target_channel: String,  payload: String, start: String) -> Message {
        Message {
            source_channel,
            target_channel,
            payload,
            start
        }
    }
}