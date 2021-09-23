mod error;
mod socket_instance;
mod util;

use socket_instance::*;

fn main() {
    println!("SOCKET SERVER STARTED");
    let instance = QuickSocketInstance::new();
    println!("INSTANCE INITIALIZED");
    let channel_1 = instance.create_tcp_channel(|v| {});

    loop {}
}
