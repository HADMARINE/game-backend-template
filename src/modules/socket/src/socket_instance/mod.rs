use crate::error::predeclared::QuickSocketError;
use crate::util;
use json::{object, JsonValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

use self::event::ResponseEvent;

pub mod event;

#[derive(Clone)]
pub struct TcpUdp<T, U> {
    pub tcp: T,
    pub udp: U,
}

#[derive(Clone)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

#[derive(Clone)]
pub struct Properties {
    port_range: PortRange,
}

#[derive(Clone)]
pub struct QuickSocketInstance {
    pub socket: TcpUdp<
        Arc<Mutex<HashMap<String, Arc<TcpChannel>>>>,
        Arc<Mutex<HashMap<String, Arc<UdpChannel>>>>,
    >,
    pub properties: Properties,
    pub self_instance: Option<Arc<Mutex<QuickSocketInstance>>>,
}

pub struct ChannelClient {
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

trait ChannelImpl {
    fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>>;
    fn emit_to(
        &self,
        clients: Vec<ChannelClient>,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>>;
    fn register_event_handler(
        &self,
        event: String,
        func: fn(JsonValue) -> Result<(), Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn disconnect_certain(
        &self,
        client: Vec<ChannelClient>,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>>;
    fn disconnect_all(&self) -> Result<(), Vec<Box<dyn std::error::Error>>>;
    fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct Channel<T> {
    pub registered_client: Arc<Mutex<Vec<ChannelClient>>>,
    pub instance: Arc<Mutex<T>>,
    pub channel_id: String,
    pub port: u16,
    event_handlers:
        HashMap<String, fn(JsonValue) -> Result<Option<JsonValue>, Box<QuickSocketError>>>,
    is_destroyed: bool,
    is_event_listener_on: bool,
    glob_instance: Arc<Mutex<QuickSocketInstance>>,
}

impl ChannelImpl for Channel<TcpListener> {
    fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors: Vec<Box<dyn std::error::Error>> = vec![];
        let value = object! {
            event: event.to_string(),
            data: value
        };
        let value = json::stringify(value);
        for client in self.registered_client.lock().unwrap().iter() {
            match match &client.stream {
                Some(v) => v,
                None => continue,
            }
            .write(value.as_bytes())
            {
                Ok(v) => v,
                Err(e) => {
                    errors.push(Box::new(e));
                    continue;
                }
            };
        }

        if errors.len() != 0 {
            return Err(errors);
        }

        Ok(())
    }

    fn emit_to(
        &self,
        clients: Vec<ChannelClient>,
        event: ResponseEvent,
        valuee: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        Ok(())
    }

    fn register_event_handler(
        &self,
        event: String,
        func: fn(JsonValue) -> Result<(), Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn disconnect_certain(
        &self,
        client: Vec<ChannelClient>,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        Ok(())
    }

    fn disconnect_all(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        Ok(())
    }

    fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl ChannelImpl for Channel<UdpSocket> {
    fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        todo!()
    }

    fn emit_to(
        &self,
        clients: Vec<ChannelClient>,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        todo!()
    }

    fn disconnect_certain(
        &self,
        client: Vec<ChannelClient>,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        todo!()
    }

    fn disconnect_all(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        todo!()
    }

    fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn register_event_handler(
        &self,
        event: String,
        func: fn(JsonValue) -> Result<(), Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

impl Channel<TcpListener> {}

impl Channel<UdpSocket> {}

pub type TcpChannel = Channel<TcpListener>;

pub type UdpChannel = Channel<UdpSocket>;

impl QuickSocketInstance {
    pub fn new() -> Arc<Mutex<Self>> {
        // let port: u16 = 8080;
        // let addr = format!("127.0.0.1:{}", &port);

        let tcp_channels: Arc<Mutex<HashMap<String, Arc<TcpChannel>>>> =
            Arc::new(Mutex::from(HashMap::new()));
        let udp_channels: Arc<Mutex<HashMap<String, Arc<UdpChannel>>>> =
            Arc::new(Mutex::from(HashMap::new()));

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

        let instance = QuickSocketInstance { socket, properties, self_instance: None };

        // let default_tcp_channel = TcpChannel {
        //     instance: Arc::new(Mutex::from(TcpListener::bind(&addr).unwrap())),
        //     registered_client: Arc::new(Mutex::from(vec![])),
        //     channel_id: Uuid::nil().to_string(),
        //     port,
        //     event_handlers: HashMap::new(),
        //     is_destroyed: false,
        //     glob_instance: &instance,
        //     is_event_listener_on: true,
        // };

        // // Add default TCP Channel to channel
        // match instance.socket.tcp.lock() {
        //     Ok(v) => v,
        //     Err(_) => panic!(QuickSocketError::ChannelInitializeFail),
        // }
        // .insert(
        //     default_tcp_channel.channel_id.clone(),
        //     Arc::new(default_tcp_channel),
        // );

        let instance_arced = Arc::new(Mutex::from(instance));

        let locked = instance_arced.lock().unwrap();

        locked.self_instance = Some(instance_arced.clone());

        instance_arced
    }

    fn get_vacant_port(&self, func: fn(u16) -> bool) -> Option<u16> {
        for i in self.properties.port_range.start.clone()..self.properties.port_range.end.clone() {
            if !func(i) {
                return Some(i);
            }
        }
        None
    }

    pub fn create_udp_channel(
        &self,
        setter: fn(&mut UdpChannel),
    ) -> Result<Arc<UdpChannel>, Box<dyn std::error::Error>> {
        let port = if let Some(v) = self.get_vacant_port(util::scan_port::udp) {
            v
        } else {
            return Err(Box::new(QuickSocketError::VacantPortSearchFail));
        };

        let addr = format!("127.0.0.1:{}", &port);

        let mut channel = UdpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: Arc::new(Mutex::from(UdpSocket::bind(addr)?)),
            registered_client: Arc::new(Mutex::from(vec![])),
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
            glob_instance: self.self_instance?.clone(),
            is_event_listener_on: true,
        };

        setter(&mut channel);

        let channel_id = channel.channel_id.clone();

        let mut mutex = self.socket.udp.lock()?;

        mutex.insert(channel_id.clone(), Arc::new(channel));

        let channel = match mutex.get_mut(&channel_id) {
            Some(v) => v.clone(),
            None => {
                return Err(Box::new(QuickSocketError::ChannelInitializeFail));
            }
        };

        drop(mutex);

        let channel_clone = channel.clone();

        if channel.is_event_listener_on {
            thread::spawn(move || {
                while !&channel.is_destroyed {
                    let mut buf: [u8; 65535] = [0; 65535];
                    let received = || -> Result<_, Box<dyn std::error::Error>> {
                        Ok(channel.instance.lock()?.recv_from(&mut buf)?)
                    }();
                    let (size, addr) = match received {
                        Ok(v) => v,
                        Err(e) => {
                            // Cannot emit to errored client because we don't know any data of client.
                            return;
                        }
                    };

                    let channel_closure_clone = channel.clone();

                    thread::spawn(move || {
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

    pub fn create_tcp_channel(
        &self,
        setter: fn(&mut TcpChannel),
    ) -> Result<Arc<TcpChannel>, Box<dyn std::error::Error>> {
        let port = if let Some(port) = self.get_vacant_port(util::scan_port::tcp) {
            port
        } else {
            return Err(Box::new(QuickSocketError::VacantPortSearchFail));
        };

        let addr = format!("127.0.0.1:{}", &port);

        let mut channel = TcpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: Arc::new(Mutex::from(TcpListener::bind(addr)?)),
            registered_client: Arc::new(Mutex::from(vec![])),
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
            is_event_listener_on: true,
            glob_instance: match self.self_instance {
                Some(v) => v,
                None => {
                    return Err(QuickSocketError::InstanceInitializeInvalid.to_box());
                }
            }.clone(),
        };

        setter(&mut channel);

        let channel_id = channel.channel_id.clone();

        let mut mutex = self.socket.tcp.lock()?;

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
            thread::spawn(move || {
                println!("TCP Thread spawned!");
                while !&channel.is_destroyed {
                println!("TCP While loop going");
                let instance_accepted =
                        || -> Result<(TcpStream, SocketAddr), Box<dyn std::error::Error>> {
                            Ok(channel.instance.lock()?.accept()?)
                        }();
                    let (mut stream, addr) = match instance_accepted {
                        Ok(v) => v,
                        Err(e) => {
                            return;
                        }
                    };

                    let channel_closure_clone = channel.clone();

                    thread::spawn(move || {
                        let channel = channel_closure_clone;

                        let mut str_buf = String::new();
                        let len = match stream.read_to_string(&mut str_buf) {
                            Ok(v) => v,
                            Err(e) => {
                                channel.emit_to(
                                    temp_client!(addr),
                                    ResponseEvent::Error,
                                    QuickSocketError::SocketBufferReadFail.jsonify(),
                                );
                                return;
                            }
                        };

                        println!("TCP data accepted: {}", &str_buf);


                        let msg = match json::parse(str_buf.as_str()) {
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
                        if !event.is_string() {
                            channel.emit_to(
                                temp_client!(addr),
                                ResponseEvent::Error,
                                QuickSocketError::JsonFormatInvalid.jsonify(),
                            );
                            return;
                        }

                        let event_handler = match channel.event_handlers.get(&event.to_string()) {
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
                                    channel.emit_to(temp_client!(addr), ResponseEvent::Ok, value);
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
                    });
                }
            });
        }

        println!("TCP Channel opened on port : {}", channel_clone.port);

        Ok(channel_clone)
    }
}
