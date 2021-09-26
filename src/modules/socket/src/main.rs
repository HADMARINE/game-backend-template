mod error;
mod socket_instance;
mod util;

use crate::error::predeclared::QuickSocketError;
use json::JsonValue;
use socket_instance::*;
use std::sync::Arc;

fn main() {
    println!("SOCKET SERVER STARTED");
    let instance = QuickSocketInstance::new();
    println!("INSTANCE INITIALIZED");
    let lock_instance = instance.write().unwrap();
    let tcp_channel_1 = lock_instance.create_tcp_channel(|v| {}).unwrap();
    tcp_channel_1
        .register_event_handler("hello".to_string(), tcp_1_hello)
        .unwrap();
    // let channel_2 = lock_instance.create_tcp_channel(|v| {});
    let udp_channel_1 = lock_instance.create_udp_channel(|v| {});
    drop(lock_instance);
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
