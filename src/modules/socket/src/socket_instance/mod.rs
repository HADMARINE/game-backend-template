use crate::error::predeclared::QuickSocketError;
use crate::util;
use json::{object, JsonValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use tungstenite::{accept, Message, WebSocket};
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
    pub self_instance: Option<Arc<RwLock<QuickSocketInstance>>>,
}

pub struct ChannelClient {
    uid: String,
    addr: SocketAddr,
    stream: Option<WebSocket<TcpStream>>,
}

impl ChannelClient {
    pub fn new(addr: SocketAddr, stream: Option<TcpStream>) -> Self {
        ChannelClient {
            addr,
            stream: match stream {
                Some(v) => Some(match accept(v) {
                    Ok(v) => v,
                    Err(e) => panic!(e),
                }),
                None => None,
            },
            uid: Uuid::new_v4().to_string(),
        }
    }
}

macro_rules! temp_client {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(ChannelClient{addr:$x,stream:None, uid:String::new() });
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
    glob_instance: Arc<RwLock<QuickSocketInstance>>,
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
        for client in self.registered_client.lock().unwrap().iter_mut() {
            match match &mut client.stream {
                Some(v) => v,
                None => continue,
            }
            .write_message(Message::Text(value.clone()))
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
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        println!("{} : {}", event.to_string(), &value);

        let mut errors: Vec<Box<dyn std::error::Error>> = vec![];

        let json_value = object! {
            event: event.to_string(),
            data: value
        };

        for client in clients {
            let mut v = match client.stream {
                Some(v) => v,
                None => {
                    errors.push(QuickSocketError::ClientDataInvalid.to_box());
                    continue;
                }
            };
            match v.write_message(Message::Text(json_value.to_string())) {
                Ok(_) => (),
                Err(e) => {
                    errors.push(Box::new(e));
                    continue;
                }
            };
        }
        if errors.len() == 0 {
            Ok(())
        } else {
            println!("Error!");
            for er in &errors {
                eprintln!("Error : {}", er);
            }
            Err(errors)
        }
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
        let mut errors: Vec<Box<dyn std::error::Error>> = vec![];
        let value = object! {
            event: event.to_string(),
            data: value
        };
        let value = json::stringify(value);
        for client in self.registered_client.lock().unwrap().iter_mut() {
            match match &mut client.stream {
                Some(v) => v,
                None => continue,
            }
            .write_message(Message::Text(value.clone()))
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
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut errors: Vec<Box<dyn std::error::Error>> = vec![];
        let value = object! {
            event: event.to_string(),
            data: value
        };
        let value = json::stringify(value);
        for mut client in clients {
            match match &mut client.stream {
                Some(v) => v,
                None => continue,
            }
            .write_message(Message::Text(value.clone()))
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

    fn disconnect_certain(
        &self,
        clients: Vec<ChannelClient>,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        todo!()
        // // let mut errors: Vec<Box<dyn std::error::Error>> = vec![];
        // let locked = self.registered_client.lock().unwrap();
        // //     locked.into_iter().find(|x| {
        // //         clients.into_iter().find()
        // //     });
        // let Some(v) = locked.get(0);

        // let Some(v) = v.stream;
        // v.s
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
    pub fn new() -> Arc<RwLock<Self>> {
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

        let instance = QuickSocketInstance {
            socket,
            properties,
            self_instance: None,
        };

        let instance_arced = Arc::new(RwLock::from(instance));

        let locked = &mut instance_arced.write().unwrap();

        locked.self_instance = Some(instance_arced.clone());

        drop(locked);

        instance_arced.clone()
    }

    fn get_vacant_port(&self, func: fn(u16) -> bool) -> Option<u16> {
        for i in self.properties.port_range.start..self.properties.port_range.end {
            if !func(i) {
                return Some(0);
            }
        }
        None
    }

    pub fn create_udp_channel(
        &self,
        setter: fn(&mut UdpChannel),
    ) -> Result<Arc<UdpChannel>, Box<dyn std::error::Error>> {
        let addr = "127.0.0.1:0";

        let sock_ins = Arc::new(Mutex::from(UdpSocket::bind(addr)?));

        let locked_temp = sock_ins.clone();

        let locked_listener = match locked_temp.lock() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let port = locked_listener.local_addr()?.port();

        drop(locked_listener);

        let mut channel = UdpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: sock_ins,
            registered_client: Arc::new(Mutex::from(vec![])),
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
            glob_instance: match self.self_instance.clone() {
                Some(v) => v,
                None => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
            }
            .clone(),
            is_event_listener_on: true,
        };

        setter(&mut channel);

        let channel_id = channel.channel_id.clone();

        let mut mutex = match self.socket.udp.lock() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

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
                println!("UDP Thread spawned!");
                while !&channel.is_destroyed {
                    println!("UDP While loop going");
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

        println!("UDP Channel opened on port : {}", channel_clone.port);

        Ok(channel_clone)
    }

    pub fn create_tcp_channel(
        &self,
        setter: fn(&mut TcpChannel),
    ) -> Result<Arc<TcpChannel>, Box<dyn std::error::Error>> {
        let addr = "127.0.0.1:0";

        let sock_ins = Arc::new(Mutex::from(TcpListener::bind(addr)?));

        let locked_tmp = sock_ins.clone();

        let locked_listener = match locked_tmp.lock() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let port = locked_listener.local_addr()?.port();

        drop(locked_listener);

        let mut channel = TcpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: sock_ins,
            registered_client: Arc::new(Mutex::from(vec![])),
            port,
            event_handlers: HashMap::new(),
            is_destroyed: false,
            is_event_listener_on: true,
            glob_instance: match self.self_instance.clone() {
                Some(v) => v,
                None => {
                    return Err(QuickSocketError::InstanceInitializeInvalid.to_box());
                }
            },
        };

        setter(&mut channel);

        let channel_id = channel.channel_id.clone();

        let mut mutex = match self.socket.tcp.lock() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        mutex.insert(channel_id.clone(), Arc::new(channel));

        let channel = match mutex.get_mut(&channel_id) {
            Some(v) => v.clone(),
            None => {
                return Err(Box::new(QuickSocketError::ChannelInitializeFail));
            }
        };

        let channel_clone = channel.clone();

        if *&channel.is_event_listener_on {
            thread::spawn(move || {
                println!("TCP Thread spawned!");
                for instance in channel.instance.lock().unwrap().incoming() {
                    if channel.is_destroyed {
                        break;
                    }
                    println!("TCP For loop going");
                    // let instance_accepted = || -> Result<_, Box<dyn std::error::Error>> {
                    //     Ok(channel.instance.lock()?.incoming())
                    // }();
                    // let mut v = match instance_accepted {
                    //     Ok(v) => v,
                    //     Err(e) => {
                    //         return;
                    //     }
                    // };

                    let instance = match instance {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let addr = match instance.local_addr() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    // let stream = match accept(instance) {
                    //     Ok(v) => v,
                    //     Err(_) => continue,
                    // };

                    let mut accepted_client = ChannelClient::new(addr, Some(instance));

                    let channel_closure_clone = channel.clone();

                    thread::spawn(move || {
                        let channel = channel_closure_clone;

                        let val = match match &mut accepted_client.stream {
                            Some(v) => v,
                            None => return,
                        }
                        .read_message()
                        {
                            Ok(v) => {
                                // let Message(res) = v;
                                match v.into_text() {
                                    Ok(v) => v,
                                    Err(e) => {
                                        channel.emit_to(
                                            vec![accepted_client],
                                            ResponseEvent::Error,
                                            QuickSocketError::SocketBufferReadFail.jsonify(),
                                        );
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                channel.emit_to(
                                    vec![accepted_client],
                                    ResponseEvent::Error,
                                    QuickSocketError::SocketBufferReadFail.jsonify(),
                                );
                                return;
                            }
                        };

                        println!("TCP data accepted: {}", &val);

                        let msg = match json::parse(&val) {
                            Ok(v) => v,
                            Err(_) => {
                                channel.emit_to(
                                    vec![accepted_client],
                                    ResponseEvent::Error,
                                    QuickSocketError::JsonParseFail.jsonify(),
                                );
                                return;
                            }
                        };

                        let event = &msg["event"];
                        if !event.is_string() {
                            channel.emit_to(
                                vec![accepted_client],
                                ResponseEvent::Error,
                                QuickSocketError::JsonFormatInvalid.jsonify(),
                            );
                            return;
                        }

                        let event_handler = match channel.event_handlers.get(&event.to_string()) {
                            Some(v) => v,
                            None => {
                                channel.emit_to(
                                    vec![accepted_client],
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
                                        vec![accepted_client],
                                        ResponseEvent::Ok,
                                        value,
                                    );
                                }
                                ()
                            }
                            Err(e) => {
                                channel.emit_to(
                                    vec![accepted_client],
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

        drop(mutex);

        Ok(channel_clone)
    }
}
