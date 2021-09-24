mod error;
mod socket_instance;
mod util;

use socket_instance::*;

fn main() {
    println!("SOCKET SERVER STARTED");
    let instance = QuickSocketInstance::new();
    println!("INSTANCE INITIALIZED");
    let lock_instance = instance.write().unwrap();
    let channel_1 = lock_instance.create_tcp_channel(|v| {});
    let channel_2 = lock_instance.create_tcp_channel(|v| {});
    if let Err(e) = channel_2 {
        println!("{}", e.to_string());
    };
    let udp_channel_1 = lock_instance.create_udp_channel(|v| {});
    drop(lock_instance);
    loop {}
}
