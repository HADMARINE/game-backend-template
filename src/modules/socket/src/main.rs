mod error;
mod socket_instance;
mod util;

use socket_instance::*;

fn main() {
    println!("SOCKET SERVER STARTED");
    let instance = QuickSocketInstance::new();
    println!("INSTANCE INITIALIZED");
    let lock_instance = instance.write().unwrap();
    let tcp_channel_1 = lock_instance.create_tcp_channel(|v| {});
    tcp_channel_1
        .unwrap()
        .register_event_handler("hello".to_string(), |value| {
            println!("Hello world from 'hello' event handler");
            Ok(None)
        })
        .unwrap();
    // let channel_2 = lock_instance.create_tcp_channel(|v| {});
    let udp_channel_1 = lock_instance.create_udp_channel(|v| {});
    drop(lock_instance);
    loop {}
}
