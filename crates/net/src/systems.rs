use std::io;

use bevy::prelude::*;

use crate::{message::Message, HeartbeatTimer, Socket};

use super::{events::NetworkEvent, transport::Transport, NetworkResource};

pub fn client_recv_packet_system(socket: Res<Socket>, mut events: EventWriter<NetworkEvent>) {
    loop {
        let mut buf = [0; 512];
        match socket.recv_from(&mut buf) {
            Ok((recv_len, address)) => {
                if let Ok(message) = serde_json::from_slice::<'_, Message>(&buf[..recv_len]) {
                    events.send(NetworkEvent::Message(address, message));
                }
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    events.send(NetworkEvent::RecvError(e));
                }
                // break loop when no messages are left to read this frame
                break;
            }
        }
    }
}

pub fn server_recv_packet_system(
    time: Res<Time>,
    socket: Res<Socket>,
    mut events: EventWriter<NetworkEvent>,
    mut net: ResMut<NetworkResource>,
) {
    loop {
        let mut buf = [0; 512];
        match socket.recv_from(&mut buf) {
            Ok((recv_len, address)) => {
                if net.connections.insert(address, time.elapsed()).is_none() {
                    // connection established
                    events.send(NetworkEvent::Connected(address));
                }

                if let Ok(message) = serde_json::from_slice::<'_, Message>(&buf[..recv_len]) {
                    events.send(NetworkEvent::Message(address, message));
                }
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    events.send(NetworkEvent::RecvError(e));
                }
                // break loop when no messages are left to read this frame
                break;
            }
        }
    }
}

pub fn send_packet_system(
    socket: Res<Socket>,
    mut events: EventWriter<NetworkEvent>,
    mut transport: ResMut<Transport>,
) {
    let messages = transport.drain_messages_to_send(|_| true);
    for message in messages {
        let result = match message.destination {
            Some(addr) => socket.send_to(&message.payload, addr),
            None => socket.send(&message.payload),
        };

        if let Err(e) = result {
            events.send(NetworkEvent::SendError(e, message));
        }
    }
}

pub fn idle_timeout_system(
    time: Res<Time>,
    mut net: ResMut<NetworkResource>,
    mut events: EventWriter<NetworkEvent>,
) {
    let idle_timeout = net.idle_timeout.clone();
    net.connections.retain(|addr, last_update| {
        let reached_idle_timeout = time.elapsed() - *last_update > idle_timeout;
        if reached_idle_timeout {
            events.send(NetworkEvent::Disconnected(*addr));
        }
        !reached_idle_timeout
    });
}

pub fn auto_heartbeat_system(
    time: Res<Time>,
    mut timer: ResMut<HeartbeatTimer>,
    mut transport: ResMut<Transport>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        transport.send(Message::Heartbeat);
    }
}
