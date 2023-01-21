use std::{collections::VecDeque, net::SocketAddr};

use bevy::prelude::Resource;

use crate::message::Message;

use super::message::OutgoingMessage;

/// Resource serving as the owner of the queue of messages to be sent. This resource also serves
/// as the interface for other systems to send messages.
#[derive(Resource)]
pub struct Transport {
    messages: VecDeque<OutgoingMessage>,
}

impl Transport {
    /// Creates a new `Transport`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    /// Creates a `OutgoingMessage` with the default guarantees provided by the `Socket` implementation and
    /// pushes it onto the messages queue to be sent on the next frame.
    pub fn send(&mut self, message: Message) {
        let message = OutgoingMessage::new(message);
        self.messages.push_back(message);
    }

    pub fn send_to(&mut self, addr: SocketAddr, message: Message) {
        let message = OutgoingMessage::new_directed(addr, message);
        self.messages.push_back(message);
    }

    /// Returns true if there are messages enqueued to be sent.
    #[must_use]
    pub fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }

    /// Returns a reference to the owned messages.
    #[must_use]
    pub fn get_messages(&self) -> &VecDeque<OutgoingMessage> {
        &self.messages
    }

    /// Drains the messages queue and returns the drained messages. The filter allows you to drain
    /// only messages that adhere to your filter. This might be useful in a scenario like draining
    /// messages with a particular urgency requirement.
    pub fn drain_messages_to_send(
        &mut self,
        mut filter: impl FnMut(&mut OutgoingMessage) -> bool,
    ) -> Vec<OutgoingMessage> {
        let mut drained = Vec::with_capacity(self.messages.len());
        let mut i = 0;
        while i != self.messages.len() {
            if filter(&mut self.messages[i]) {
                if let Some(m) = self.messages.remove(i) {
                    drained.push(m);
                }
            } else {
                i += 1;
            }
        }
        drained
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct TestPayload {
        x: f32,
    }

    #[derive(Serialize)]
    struct HeartbeatPayload;

    #[test]
    fn test_send_to() {
        let mut transport = create_test_transport();

        transport.send_to("127.0.0.1:3000".parse().unwrap(), &test_payload());

        let packet = &transport.messages[0];

        assert_eq!(transport.messages.len(), 1);
        assert_eq!(packet.payload, serde_json::to_vec(&test_payload()).unwrap());
    }

    #[test]
    fn test_has_messages() {
        let mut transport = create_test_transport();
        assert_eq!(transport.has_messages(), false);
        transport.send_to("127.0.0.1:3000".parse().unwrap(), &test_payload());
        assert_eq!(transport.has_messages(), true);
    }

    #[test]
    fn test_drain_only_heartbeat_messages() {
        let mut transport = create_test_transport();

        let addr = "127.0.0.1:3000".parse().unwrap();
        transport.send_to(addr, &test_payload());
        transport.send_to(addr, &HeartbeatPayload);
        transport.send_to(addr, &test_payload());
        transport.send_to(addr, &HeartbeatPayload);
        transport.send_to(addr, &test_payload());

        assert_eq!(
            transport
                .drain_messages_to_send(
                    |m| m.payload == serde_json::to_vec(&HeartbeatPayload).unwrap()
                )
                .len(),
            2
        );
        // validate removal
        assert_eq!(
            transport
                .drain_messages_to_send(
                    |m| m.payload == serde_json::to_vec(&HeartbeatPayload).unwrap()
                )
                .len(),
            0
        );
        assert_eq!(transport.drain_messages_to_send(|_| false).len(), 0);
        assert_eq!(transport.drain_messages_to_send(|_| true).len(), 3);
        // validate removal
        assert_eq!(transport.drain_messages_to_send(|_| true).len(), 0);
    }

    fn test_payload() -> TestPayload {
        TestPayload { x: 1. }
    }

    fn create_test_transport() -> Transport {
        Transport::new()
    }
}
