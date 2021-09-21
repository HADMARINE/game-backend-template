use crate::error;
use crate::error::predeclared::QuickSocketError;
use crate::util;
use async_trait::async_trait;
use futures::prelude::*;
use json::JsonValue;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::{convert::TryInto, net};
use tokio::net::TcpStream;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::Mutex;
use tokio_serde::formats::SymmetricalJson;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use uuid::Uuid;

use self::event::ResponseEvent;

mod event;

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
    socket: TcpUdp<
        Arc<Mutex<HashMap<String, Arc<TcpChannel>>>>,
        Arc<Mutex<HashMap<String, Arc<UdpChannel>>>>,
    >,
    properties: Properties,
}

struct ChannelClient {
    uid: Option<String>,
    addr: SocketAddr,
    stream: Option<TcpStream>,
}

macro_rules! temp_client {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(ChannelClient{addr:$x,stream:None, uid:None });
            )*
            temp_vec
        }
    };
}

#[async_trait]
trait ChannelImpl {
    async fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn emit_to(
        &self,
        clients: Vec<ChannelClient>,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn register_event_handler(
        &self,
        event: String,
        func: fn(JsonValue) -> Result<(), Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn disconnect_certain(
        &self,
        client: Vec<ChannelClient>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn disconnect_all(&self) -> Result<(), Box<dyn std::error::Error>>;
    async fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>>;
}

struct Channel<T> {
    pub registered_client: Arc<Mutex<Vec<ChannelClient>>>,
    pub instance: Arc<Mutex<T>>,
    pub channel_id: String,
    pub port: u16,
    event_handlers:
        HashMap<String, fn(JsonValue) -> Result<Option<JsonValue>, Box<QuickSocketError>>>,
    is_destroyed: bool,
    is_event_listener_on: bool,
    glob_instance: &'static QuickSocketInstance,
}

#[async_trait]
impl ChannelImpl for Channel<TcpListener> {
    // async fn listen(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     let (stream, addr) = self.instance.accept().await.unwrap();

    //     let length_delimited = FramedRead::new(stream, LengthDelimitedCodec::new());

    //     let mut deserialized = tokio_serde::SymmetricallyFramed::new(
    //         length_delimited,
    //         SymmetricalJson::<Value>::default(),
    //     );

    //     tokio::spawn(async move {
    //         while let Some(msg) = deserialized.try_next().await.unwrap() {
    //             println!("Got : {:?}", msg);
    //         }
    //     });

    //     Ok(())
    // }

    async fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
        // for client in &self.registered_client {
        //     // client.addr.
        // }

        // Ok(())
    }

    async fn emit_to(
        &self,
        clients: Vec<ChannelClient>,
        event: ResponseEvent,
        valuee: JsonValue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn disconnect_certain(
        &self,
        client: Vec<ChannelClient>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn disconnect_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.is_destroyed = true;
        Ok(())
    }

    async fn register_event_handler(
        &self,
        event: String,
        func: fn(JsonValue) -> Result<(), Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

#[async_trait]
impl ChannelImpl for Channel<UdpSocket> {
    // async fn listen(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut buf: [u8; 65535] = [0; 65535];

    //     let (length, addr) = self.instance.recv_from(&mut buf).await?;

    //     let mut length_delimited = FramedRead::new(stream, LengthDelimitedCodec::new());

    //     let mut deserialized = tokio_serde::SymmetricallyFramed::new(
    //         length_delimited,
    //         SymmetricalJson::<Value>::default(),
    //     );

    //     tokio::spawn(async move {
    //         while let Some(msg) = deserialized.try_next().await.unwrap() {
    //             println!("Got : {:?}", msg);
    //         }
    //     });

    //     Ok(())
    // }

    async fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn emit_to(
        &self,
        clients: Vec<ChannelClient>,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn disconnect_certain(
        &self,
        client: Vec<ChannelClient>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn disconnect_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn register_event_handler(
        &self,
        event: String,
        func: fn(JsonValue) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

impl Channel<TcpListener> {}

impl Channel<UdpSocket> {}

type TcpChannel = Channel<TcpListener>;

type UdpChannel = Channel<UdpSocket>;

impl QuickSocketInstance {
    pub async fn new(&'static mut self) -> Result<QuickSocketInstance, Box<dyn std::error::Error>> {
        use tokio::net::*;

        let port: u16 = 8080;
        let addr = format!("127.0.0.1:{}", &port);

        let default_tcp_channel = TcpChannel {
            instance: Arc::new(Mutex::from(TcpListener::bind(&addr).await?)),
            registered_client: Arc::new(Mutex::from(vec![])),
            channel_id: Uuid::nil().to_string(),
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
            glob_instance: self,
            is_event_listener_on: true,
        };

        let tcp_channels: Arc<Mutex<HashMap<String, Arc<TcpChannel>>>> =
            Arc::new(Mutex::from(HashMap::new()));
        let udp_channels: Arc<Mutex<HashMap<String, Arc<UdpChannel>>>> =
            Arc::new(Mutex::from(HashMap::new()));

        // Add default TCP Channel to channel
        tcp_channels.try_lock()?.insert(
            default_tcp_channel.channel_id.clone(),
            Arc::new(default_tcp_channel),
        );

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
        &'static self,
        setter: fn(&mut UdpChannel),
    ) -> Result<Arc<UdpChannel>, Box<dyn std::error::Error>> {
        let port = if let Some(v) = self.get_vacant_port(util::scan_port::udp) {
            v
        } else {
            return Err(Box::new(
                error::predeclared::QuickSocketError::VacantPortSearchFail,
            ));
        };

        let addr = format!("127.0.0.1:{}", &port);

        let mut channel = UdpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: Arc::new(Mutex::from(UdpSocket::bind(addr).await?)),
            registered_client: Arc::new(Mutex::from(vec![])),
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
            glob_instance: self,
            is_event_listener_on: true,
        };

        setter(&mut channel);

        let channel_id = channel.channel_id.clone();

        let mut mutex = self.socket.udp.lock().await;

        mutex.insert(channel_id.clone(), Arc::new(channel));

        let channel = match mutex.get_mut(&channel_id) {
            Some(v) => v.clone(),
            None => {
                return Err(Box::new(
                    error::predeclared::QuickSocketError::ChannelInitializeFail,
                ));
            }
        };

        drop(mutex);

        let channel_clone = channel.clone();

        if channel.is_event_listener_on {
            tokio::spawn(async move {
                while !&channel.is_destroyed {
                    let mut buf: [u8; 65535] = [0; 65535];
                    let (size, addr) = channel
                        .instance
                        .lock()
                        .await
                        .recv_from(&mut buf)
                        .await
                        .unwrap();

                    let channel_closure_clone = channel.clone();

                    tokio::spawn(async move {
                        let buf = &mut buf[..size];
                        let channel = channel_closure_clone;

                        if let Ok(value) = std::str::from_utf8(buf) {
                            let msg = match json::parse(value) {
                                Ok(v) => v,
                                Err(e) => {
                                    channel.emit_to(
                                        temp_client!(addr),
                                        ResponseEvent::Error,
                                        QuickSocketError::JsonParseFail.jsonify(),
                                    );
                                    return;
                                }
                            };

                            let event = &msg["event"];
                            let data = &msg["data"];

                            if !event.is_string() {
                                channel.emit_to(
                                    temp_client!(addr),
                                    ResponseEvent::Error,
                                    QuickSocketError::JsonFormatInvalid.jsonify(),
                                );
                                return;
                            }

                            let event_handler = match channel.event_handlers.get(&event.to_string())
                            {
                                Some(v) => v,
                                None => {
                                    channel.emit_to(
                                        temp_client!(addr),
                                        ResponseEvent::Error,
                                        QuickSocketError::EventNotFound.jsonify(),
                                    );
                                    return;
                                }
                            };

                            match event_handler(msg["data"].to_owned()) {
                                Ok(v) => {
                                    if let Some(value) = v {
                                        channel.emit_to(
                                            temp_client!(addr),
                                            ResponseEvent::Ok,
                                            value,
                                        );
                                    };
                                    ()
                                }
                                Err(e) => {
                                    channel.emit_to(
                                        temp_client!(addr),
                                        ResponseEvent::Error,
                                        e.jsonify(),
                                    );
                                    ()
                                }
                            }
                        } else {
                            channel.emit_to(
                                temp_client!(addr),
                                ResponseEvent::Error,
                                QuickSocketError::InternalServerError.jsonify(),
                            );
                            return;
                        }
                    });
                }
            });
        }

        Ok(channel_clone)
    }

    pub async fn create_tcp_channel(
        &'static mut self,
        setter: fn(&mut TcpChannel),
    ) -> Result<Arc<TcpChannel>, Box<dyn std::error::Error>> {
        let port = if let Some(port) = self.get_vacant_port(util::scan_port::tcp) {
            port
        } else {
            return Err(Box::new(
                error::predeclared::QuickSocketError::VacantPortSearchFail,
            ));
        };

        let addr = format!("127.0.0.1:{}", &port);

        let mut channel = TcpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: Arc::new(Mutex::from(TcpListener::bind(addr).await?)),
            registered_client: Arc::new(Mutex::from(vec![])),
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
            is_event_listener_on: true,
            glob_instance: self,
        };

        setter(&mut channel);

        let channel_id = channel.channel_id.clone();

        let mut mutex = self.socket.tcp.lock().await;

        mutex.insert(channel_id.clone(), Arc::new(channel));

        let channel = match mutex.get_mut(&channel_id) {
            Some(v) => v.clone(),
            None => {
                return Err(Box::new(QuickSocketError::ChannelInitializeFail));
            }
        };

        drop(mutex);

        let channel_clone = channel.clone();

        if *&channel.is_event_listener_on {
            tokio::spawn(async move {
                while !&channel.is_destroyed {
                    let (stream, addr) = channel.instance.lock().await.accept().await.unwrap();

                    let length_delimited = FramedRead::new(stream, LengthDelimitedCodec::new());

                    let mut deserialized = tokio_serde::SymmetricallyFramed::new(
                        length_delimited,
                        SymmetricalJson::<Value>::default(),
                    );

                    let channel_closure_clone = channel.clone();

                    tokio::spawn(async move {
                        let channel = channel_closure_clone;

                        while let Some(msg) = match deserialized.try_next().await {
                            Ok(v) => v,
                            Err(e) => {
                                channel.emit_to(
                                    temp_client!(addr),
                                    ResponseEvent::Error,
                                    QuickSocketError::JsonParseFail.jsonify(),
                                );
                                None
                            }
                        } {
                            let event = &msg["event"];
                            let data = &msg["data"];
                            if !event.is_string() {
                                channel.emit_to(
                                    temp_client!(addr),
                                    ResponseEvent::Error,
                                    QuickSocketError::JsonFormatInvalid.jsonify(),
                                );
                                return;
                            }

                            let event_handler = match channel.event_handlers.get(&event.to_string())
                            {
                                Some(v) => v,
                                None => {
                                    channel.emit_to(
                                        temp_client!(addr),
                                        ResponseEvent::Error,
                                        QuickSocketError::EventNotFound.jsonify(),
                                    );
                                    return;
                                }
                            };

                            match event_handler(msg["data"].to_owned()) {
                                Ok(v) => {
                                    if let Some(value) = v {
                                        channel.emit_to(
                                            temp_client!(addr),
                                            ResponseEvent::Ok,
                                            value,
                                        );
                                    }
                                    ()
                                }
                                Err(e) => {
                                    channel.emit_to(
                                        temp_client!(addr),
                                        ResponseEvent::Error,
                                        e.jsonify(),
                                    );
                                    ()
                                }
                            }
                        }
                    });
                }
            });
        }

        Ok(channel_clone)
    }
}

// fn listen(socket: &net::UdpSocket, mut buffer: &mut [u8]) -> usize {
//     let (number_of_bytes, src_addr) = socket.recv_from(&mut buffer).expect("No data recieved");

//     println!("{:?}", number_of_bytes);
//     println!("{:?}", src_addr);

//     number_of_bytes
// }
