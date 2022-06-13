use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum Message {
    JsonValue(serde_json::Value),
}

pub type MessageSender = broadcast::Sender<Message>;
