use bevy::prelude::Vec3;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Heartbeat,
    Positional(Vec3),
}

pub struct OutgoingMessage {
    /// The serialized payload itself.
    pub payload: Vec<u8>,
    pub destination: Option<SocketAddr>,
}

impl OutgoingMessage {
    /// Creates and returns a new Message.
    pub(crate) fn new<P: Serialize>(payload: P) -> Self {
        Self {
            payload: serde_json::to_vec(&payload).unwrap_or_default(),
            destination: None,
        }
    }

    /// Creates and returns a new Messaged directed to a specfic Address.
    pub(crate) fn new_directed<P: Serialize>(addr: SocketAddr, payload: P) -> Self {
        Self {
            payload: serde_json::to_vec(&payload).unwrap_or_default(),
            destination: Some(addr),
        }
    }
}
