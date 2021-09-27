mod error;
mod socket_instance;
mod util;

use crate::{error::predeclared::QuickSocketError, socket_instance::event::ResponseEvent};
use json::{object, JsonValue};
use socket_instance::*;
use std::{sync::Arc, thread, time::Duration};

fn main() {
    println!("SOCKET SERVER STARTED");
    let instance = QuickSocketInstance::new();
    println!("INSTANCE INITIALIZED");
    let lock_instance = instance.write().unwrap();
    let tcp_channel_1 = lock_instance.create_tcp_channel(|v| {}).unwrap();
    tcp_channel_1
        .register_event_handler("hello".to_string(), tcp_1_hello)
        .unwrap();
    tcp_channel_1.register_event_handler("register".to_string(), register);
    let udp_channel_1 = lock_instance.create_udp_channel(|v| {}).unwrap();
    udp_channel_1
        .register_event_handler("hello".to_string(), tcp_1_hello)
        .unwrap();
    let tcp_channel_1_clone = tcp_channel_1.clone();

    drop(lock_instance);
    thread::spawn(move || loop {
        tcp_channel_1_clone
            .emit_all(
                ResponseEvent::Data,
                object! {
                    event:"world_data".to_string(),
                    data: [123,123,123,0]
                },
            )
            .unwrap();
        thread::sleep(Duration::from_secs(1));
    });
    loop {}
}

fn tcp_1_hello(
    ch: Arc<dyn ChannelImpl>,
    v: JsonValue,
    c: ChannelClient,
) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    println!("Hello world from 'hello' event handler");
    ch.emit_to(vec![c], event::ResponseEvent::Ok, JsonValue::Null);
    Ok(None)
}

fn register(
    ch: Arc<dyn ChannelImpl>,
    v: JsonValue,
    c: ChannelClient,
) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    println!("register");
    ch.register_client(c).unwrap();
    // ch.emit_to(c, event::ResponseEvent::Ok, JsonValue::Null);
    Ok(None)
}
