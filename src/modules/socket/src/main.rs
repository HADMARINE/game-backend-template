mod app;
mod error;
mod socket_instance;
mod util;

use crate::{error::predeclared::QuickSocketError, socket_instance::event::ResponseStatus};
use json::{object, JsonValue};
use socket_instance::*;
use std::{sync::Arc, thread, time::Duration};

fn main() {
    // println!("SOCKET SERVER STARTED");
    // let instance = QuickSocketInstance::new();
    // println!("INSTANCE INITIALIZED");
    // let lock_instance = instance.write().unwrap();
    // let tcp_channel_1 = lock_instance
    //     .create_tcp_channel(
    //         |v| {},
    //         TcpChannelCreatePreferences {
    //             concurrent: true,
    //             delete_client_when_closed: false,
    //         },
    //     )
    //     .unwrap();
    // let tcp_channel_2 = lock_instance
    //     .create_tcp_channel(
    //         |v| {},
    //         TcpChannelCreatePreferences {
    //             concurrent: false,
    //             delete_client_when_closed: false,
    //         },
    //     )
    //     .unwrap();

    // tcp_channel_1
    //     .register_event_handler("hello".to_string(), tcp_1_hello)
    //     .unwrap();
    // tcp_channel_1
    //     .register_event_handler("register".to_string(), register)
    //     .unwrap();
    // tcp_channel_1
    //     .register_event_handler("deregister".to_string(), deregister)
    //     .unwrap();
    // tcp_channel_2
    //     .register_event_handler("register".to_string(), register)
    //     .unwrap();
    // let udp_channel_1 = lock_instance
    //     .create_udp_channel(|v| {}, UdpChannelCreatePreferences::default())
    //     .unwrap();
    // udp_channel_1
    //     .register_event_handler("hello".to_string(), tcp_1_hello)
    //     .unwrap();
    // udp_channel_1
    //     .register_event_handler("register".to_string(), register)
    //     .unwrap();
    // let tcp_channel_1_clone = tcp_channel_1.clone();
    // let tcp_channel_2_clone = tcp_channel_2.clone();
    // let udp_channel_1_clone = udp_channel_1.clone();

    // drop(lock_instance);
    // thread::spawn(move || loop {
    //     tcp_channel_1_clone
    //         .emit_all(
    //             "world_data".to_string(),
    //             ResponseStatus::Data,
    //             JsonValue::Array(vec![123.into(), 123.into(), 123.into(), 0.into()]),
    //         )
    //         .unwrap();
    //     tcp_channel_2_clone
    //         .emit_all(
    //             "world_data".to_string(),
    //             ResponseStatus::Data,
    //             JsonValue::Array(vec![123.into(), 123.into(), 123.into(), 0.into()]),
    //         )
    //         .unwrap();
    //     udp_channel_1_clone
    //         .emit_all(
    //             "world_data".to_string(),
    //             ResponseStatus::Data,
    //             JsonValue::Array(vec![123.into(), 123.into(), 123.into(), 0.into()]),
    //         )
    //         .unwrap();
    //     thread::sleep(Duration::from_secs(1));
    // });
    // loop {}
}

fn tcp_1_hello(ctrl: ChannelController) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    println!("Hello world from 'hello' event handler");
    ctrl.emit_to(
        vec![ctrl.get_client_data()],
        event::ResponseStatus::Ok,
        JsonValue::Null,
    );
    Ok(None)
}

fn register(ctrl: ChannelController) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    println!("register");
    match ctrl.register_client(ctrl.get_client_data()) {
        Ok(v) => (),
        Err(e) => {
            // match e {
            //     QuickSocketError => ch.emit_to(c, ResponseEvent::Error, e.jsonify()),
            // };
            // if e == QuickSocketError {}

            ctrl.emit_to(
                vec![ctrl.get_client_data()],
                ResponseStatus::Error,
                object! {
                    data: String::from("Client already exists!")
                },
            );
        }
    };
    // ch.emit_to(c, event::ResponseEvent::Ok, JsonValue::Null);
    Ok(None)
}

fn deregister(ctrl: ChannelController) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    println!("deregister");

    match ctrl.disconnect_certain(vec![ctrl.get_client_data()]) {
        Ok(v) => (),
        Err(e) => {
            ctrl.emit_to(
                vec![ctrl.get_client_data()],
                ResponseStatus::Error,
                object! {
                    data:String::from("Disconnect failed")
                },
            );
        }
    };

    Ok(None)
}

fn reregister(ctrl: ChannelController) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    Ok(None)
}

fn get_client_data(ctrl: ChannelController) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    Ok(None)
}
