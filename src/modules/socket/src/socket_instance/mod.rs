use crate::error;
use crate::util;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::thread;
use std::{convert::TryInto, net};
use tokio::net::{TcpListener, UdpSocket};

struct TcpUdp<T, U> {
    tcp: T,
    udp: U,
}

struct PortRange {
    start: u16,
    end: u16,
}

struct Properties {
    port_range: PortRange,
}

struct QuickSocketInstance {
    socket: TcpUdp<Vec<TcpChannel>, Vec<UdpChannel>>,
    properties: Properties,
}

struct ChannelClient {
    uuid: String,
    addr: SocketAddr,
}

trait ChannelImpl {
    fn emit_all(&self, message: String) -> Result<String, Box<dyn std::error::Error>>;
    fn emit_to<T>(
        &self,
        clients: [ChannelClient],
        message: String,
    ) -> Result<T, Box<dyn std::error::Error>>;
    fn register_event_handler<T>(
        event: String,
        func: dyn Fn(String) -> Result<T, Box<dyn std::error::Error>>,
    );
    fn disconnect_certain(&self, client: [ChannelClient])
        -> Result<(), Box<dyn std::error::Error>>;
    fn disconnect_all(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn recieve_from(&self) -> Result<Option<String>, Box<dyn std::error::Error>>;
}

struct Channel<T> {
    pub registered_client: Vec<ChannelClient>,
    pub instance: T,
    pub channel_id: i32,
    pub port: u16,
    event_handlers: HashMap<String, fn(String) -> Result<(), Box<dyn std::error::Error>>>,
    is_destroyed: bool,
}

impl<T> ChannelImpl for Channel<T> {
    // pub fn
}

impl Channel<TcpListener> {}

impl Channel<UdpSocket> {}

type TcpChannel = Channel<TcpListener>;

type UdpChannel = Channel<UdpSocket>;

impl QuickSocketInstance {
    pub async fn new() -> Result<QuickSocketInstance, Box<dyn std::error::Error>> {
        use tokio::net::*;

        let port: u16 = 8080;
        let addr = format!("127.0.0.1:{}", &port);
        let default_tcp_channel = TcpChannel {
            instance: TcpListener::bind(&addr).await?,
            registered_client: vec![],
            channel_id: 0,
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
        };

        let tcp_channels: Mutex<Vec<TcpChannel>> = vec![default_tcp_channel];
        let udp_channels: Mutex<Vec<UdpChannel>> = vec![];

        let socket = TcpUdp {
            tcp: tcp_channels,
            udp: udp_channels,
        };

        let properties = Properties {
            port_range: PortRange {
                start: 20000,
                end: 65535,
            },
        };

        let instance = QuickSocketInstance { socket, properties };

        Ok(instance)
    }

    fn get_vacant_port(&self, func: fn(u16) -> bool) -> Option<u16> {
        for i in self.properties.port_range.start.clone()..self.properties.port_range.end.clone() {
            if !func(i) {
                return Some(i);
            }
        }
        None
    }

    pub async fn create_udp_channel(
        &self,
        handler: fn(UdpChannel),
    ) -> Result<UdpChannel, Box<dyn std::error::Error>> {
        let rng = rand::thread_rng();

        let mut port = if let Some(v) = self.get_vacant_port(util::scan_port::udp) {
            v
        } else {
            return Err(Box::new(error::PortNotFoundError));
        };
        match self.get_vacant_port(util::scan_port::udp) {
            Some(v) => v,
            None => Err(),
        }

        let addr = format!("127.0.0.1:{}", &port);

        let mut channel = UdpChannel {
            channel_id: self.socket.udp.len().try_into().unwrap(),
            instance: UdpSocket::bind(addr).await?,
            registered_client: vec![],
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
        };

        // channel.event_handlers.insert("hello".to_string(), || 1);

        thread::spawn(async move {
            while !&channel.is_destroyed {
                // let mut buf = [0; 1024];
                // let (size, peer) = channel.instance.recv_from(&buf).await;
                // let buf = &mut buf[..size];
                // for client in &channel.registered_client {}
                handler(&mut channel);
            }
        });

        Ok(channel)
    }

    pub async fn create_tcp_channel(
        &self,
        handler: fn(TcpChannel),
    ) -> Result<TcpChannel, Box<dyn std::error::Error>> {
        let rng = rand::thread_rng();

        let mut port = if let Some(v) = self.get_vacant_port(util::scan_port::udp) {
            v
        } else {
            return Err(Box::new(error::PortNotFoundError));
        };
        match self.get_vacant_port(util::scan_port::udp) {
            Some(v) => v,
            None => Err(),
        }

        let addr = format!("127.0.0.1:{}", &port);

        let channel = TcpChannel {
            channel_id: self.socket.udp.len().try_into().unwrap(),
            instance: TcpListener::bind(addr).await.unwrap(),
            registered_client: vec![],
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
        };

        thread::spawn(move || {
            while !&channel.is_destroyed {
                handler(&mut channel);
            }
        });

        Ok(channel)
    }
}

fn listen(socket: &net::UdpSocket, mut buffer: &mut [u8]) -> usize {
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buffer).expect("No data recieved");

    println!("{:?}", number_of_bytes);
    println!("{:?}", src_addr);

    number_of_bytes
}
