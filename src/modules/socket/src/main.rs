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
    let tcp_channel_1 = lock_instance.create_tcp_channel(|v| {}, true).unwrap();
    let tcp_channel_2 = lock_instance.create_tcp_channel(|v| {}, false).unwrap();

    tcp_channel_1
        .register_event_handler("hello".to_string(), tcp_1_hello)
        .unwrap();
    tcp_channel_1
        .register_event_handler("register".to_string(), register)
        .unwrap();
    tcp_channel_1
        .register_event_handler("deregister".to_string(), deregister)
        .unwrap();
    tcp_channel_2
        .register_event_handler("register".to_string(), register)
        .unwrap();
    let udp_channel_1 = lock_instance.create_udp_channel(|v| {}).unwrap();
    udp_channel_1
        .register_event_handler("hello".to_string(), tcp_1_hello)
        .unwrap();
    let tcp_channel_1_clone = tcp_channel_1.clone();
    let tcp_channel_2_clone = tcp_channel_2.clone();

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
        tcp_channel_2_clone
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
    match ch.register_client(c.clone()) {
        Ok(v) => (),
        Err(e) => {
            // match e {
            //     QuickSocketError => ch.emit_to(c, ResponseEvent::Error, e.jsonify()),
            // };
            // if e == QuickSocketError {}

            ch.emit_to(
                vec![c],
                ResponseEvent::Error,
                object! {
                    data: String::from("Client already exists!")
                },
            );
        }
    };
    // ch.emit_to(c, event::ResponseEvent::Ok, JsonValue::Null);
    Ok(None)
}

fn deregister(
    ch: Arc<dyn ChannelImpl>,
    v: JsonValue,
    c: ChannelClient,
) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    println!("deregister");

    match ch.disconnect_certain(vec![c.clone()]) {
        Ok(v) => (),
        Err(e) => {
            ch.emit_to(
                vec![c],
                ResponseEvent::Error,
                object! {
                    data:String::from("Disconnect failed")
                },
            );
        }
    };

    Ok(None)
}
