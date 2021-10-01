use crate::error::predeclared::QuickSocketError;
use json::{object, JsonValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use std::{clone, net};
use tracing::field::debug;
use tracing::trace;
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

#[derive(Clone, Debug)]
pub struct ChannelClient {
    pub uid: String,
    pub addr: SocketAddr,
    pub stream: Option<Arc<RwLock<WebSocket<TcpStream>>>>,
}

impl ChannelClient {
    pub fn new(addr: SocketAddr, stream: Option<WebSocket<TcpStream>>) -> Self {
        ChannelClient {
            addr,
            stream: match stream {
                Some(v) => Some(Arc::new(RwLock::from(v))),
                None => None,
            },
            uid: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChannelCreatePreferences {
    delete_client_when_closed: bool,
    concurrent: bool,
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

pub trait ChannelImpl {
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
        func: fn(
            Arc<dyn ChannelImpl>,
            JsonValue,
            ChannelClient,
        ) -> Result<Option<JsonValue>, Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn disconnect_certain(
        &self,
        client: Vec<ChannelClient>,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>>;
    fn disconnect_all(&self) -> Result<(), Vec<Box<dyn std::error::Error>>>;
    fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn register_client(&self, client: ChannelClient) -> Result<(), Box<dyn std::error::Error>>; // cmp uid & ip & port
    fn reconnect_client_by_uid(
        &self,
        uid: String,
        client: ChannelClient,
    ) -> Result<(), Box<dyn std::error::Error>>; // cmp uid
}

pub struct Channel<T> {
    pub registered_client: Arc<RwLock<Vec<ChannelClient>>>,
    pub instance: Arc<RwLock<T>>,
    pub channel_id: String,
    pub port: u16,
    pub pref: ChannelCreatePreferences,
    event_handlers: Arc<
        RwLock<
            HashMap<
                String,
                fn(
                    Arc<dyn ChannelImpl>,
                    JsonValue,
                    ChannelClient,
                ) -> Result<Option<JsonValue>, Box<QuickSocketError>>,
            >,
        >,
    >,
    is_destroyed: Arc<RwLock<bool>>,
    is_event_listener_on: Arc<RwLock<bool>>,
    glob_instance: Arc<RwLock<QuickSocketInstance>>,
}

impl ChannelImpl for Channel<TcpListener> {
    fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let clients = match self.registered_client.read() {
            Ok(v) => v.to_vec(),
            Err(_) => return Err(vec![QuickSocketError::ClientDataInvalid.to_box()]),
        };
        match self.emit_to(clients, event, value) {
            Ok(v) => Ok(()),
            Err(e) => {
                let mut e = e;
                e.retain(|err| match err.as_ref() {
                    QuickSocketError => match &err.as_ref().downcast_ref() {
                        Some(v) => match v {
                            QuickSocketError::ConnectionClosed(client) => {
                                println!("Closing client : {}", &client.uid);
                                if self.pref.delete_client_when_closed {
                                    if self.disconnect_certain(vec![client.clone()]).is_err() {
                                        trace!(
                                            "Failed to disconnect connection (UID : {})",
                                            &client.uid
                                        );
                                    };
                                }

                                false
                            }
                            _ => true,
                        },
                        None => panic!("Error not found"),
                    },
                    _ => true,
                });

                if e.len() == 0 {
                    return Ok(());
                }

                Err(e)
            }
        }
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
            let v = match &client.stream {
                Some(v) => v,
                None => {
                    errors.push(QuickSocketError::ClientDataInvalid.to_box());
                    continue;
                }
            };
            let mut write_locked = match v.write() {
                Ok(v) => v,
                Err(_) => return Err(vec![QuickSocketError::ChannelInitializeFail.to_box()]),
            };
            match write_locked.write_message(Message::Text(json_value.to_string())) {
                Ok(_) => {
                    drop(write_locked);
                    ()
                }
                Err(e) => {
                    drop(write_locked);
                    match e {
                        tungstenite::Error::AlreadyClosed => {
                            errors
                                .push(Box::new(QuickSocketError::ConnectionClosed(client.clone())));
                        }
                        _ => {
                            errors.push(Box::new(e));
                            continue;
                        }
                    }
                }
            };
        }
        if errors.len() == 0 {
            Ok(())
        } else {
            log::debug!("Error!");
            for er in &errors {
                eprintln!("Error : {}", er);
            }
            Err(errors)
        }
    }

    fn register_event_handler(
        &self,
        event: String,
        func: fn(
            Arc<dyn ChannelImpl>,
            JsonValue,
            ChannelClient,
        ) -> Result<Option<JsonValue>, Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if match self.event_handlers.read() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        }
        .get(&event)
            != None
        {
            return Err(QuickSocketError::EventAlreadyExists.to_box());
        }
        match self.event_handlers.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        }
        .insert(event, func);
        Ok(())
    }

    fn disconnect_certain(
        &self,
        search_clients: Vec<ChannelClient>,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let registered_client_clone = self.registered_client.clone();
        let mut clients_pre = match registered_client_clone.write() {
            Ok(v) => v,
            Err(_) => return Err(vec![QuickSocketError::ClientDataInvalid.to_box()]),
        };

        let clients: &mut Vec<ChannelClient> = clients_pre.as_mut();

        let search_clients = Rc::new(RefCell::new(search_clients));

        clients.retain(|client| {
            for cmp_client in search_clients.borrow().iter() {
                if client.uid == cmp_client.uid {
                    // search_clients
                    //     .borrow_mut() // !todo : fix panicking bc of multi borrowing
                    //     .retain(|cmp_client_babe| cmp_client_babe.uid != cmp_client.uid);
                    return false;
                }
            }
            return true;
        });

        Ok(())
    }

    fn disconnect_all(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut client = match self.registered_client.write() {
            Ok(v) => v,
            Err(_) => return Err(vec![QuickSocketError::ClientDataInvalid.to_box()]),
        };

        client.clear();
        client.shrink_to_fit();

        Ok(())
    }

    fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut is_event_listener_on = match self.is_event_listener_on.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };
        *is_event_listener_on = false;
        let mut is_destroyed = match self.is_destroyed.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };
        *is_destroyed = true;
        Ok(())
    }

    fn register_client(&self, client: ChannelClient) -> Result<(), Box<dyn std::error::Error>> {
        let mut locked_client_list = match self.registered_client.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let cloned_client = client.clone();

        if locked_client_list
            .iter()
            .find(move |v| {
                if v.addr == cloned_client.addr || v.uid == cloned_client.uid {
                    return true;
                }
                false
            })
            .is_some()
        {
            return Err(QuickSocketError::ClientAlreadyExists.to_box());
        };

        locked_client_list.push(client);
        drop(locked_client_list);

        return Ok(());
    }

    fn reconnect_client_by_uid(
        &self,
        uid: String,
        new_client: ChannelClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut locked_client_list = match self.registered_client.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let found_client = locked_client_list.iter_mut().find(move |v| v.uid == uid);
        if found_client.is_some() {
            let found_client = match found_client {
                Some(v) => v,
                None => return Err(QuickSocketError::ClientNotRegistered.to_box()),
            };
            std::mem::replace(found_client, new_client);
        };
        Ok(())
    }
}

impl ChannelImpl for Channel<UdpSocket> {
    fn emit_all(
        &self,
        event: ResponseEvent,
        value: JsonValue,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let locked_registered_client = match self.registered_client.read() {
            Ok(v) => v.to_vec(),
            Err(_) => return Err(vec![QuickSocketError::ClientDataInvalid.to_box()]),
        };
        self.emit_to(locked_registered_client, event, value)
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
        for client in clients {
            match match self.instance.read() {
                Ok(v) => v,
                Err(_) => {
                    errors.push(QuickSocketError::ClientDataInvalid.to_box());
                    continue;
                }
            }
            .send_to(value.as_bytes(), client.addr)
            {
                Ok(_) => continue,
                Err(_) => errors.push(QuickSocketError::DataResponseFail.to_box()),
            }
        }

        if errors.len() != 0 {
            return Err(errors);
        }

        Ok(())
    }

    fn disconnect_certain(
        &self,
        search_clients: Vec<ChannelClient>,
    ) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let registered_client_clone = self.registered_client.clone();
        let mut clients_pre = match registered_client_clone.write() {
            Ok(v) => v,
            Err(_) => return Err(vec![QuickSocketError::ClientDataInvalid.to_box()]),
        };

        let clients: &mut Vec<ChannelClient> = clients_pre.as_mut();

        let search_clients = Rc::new(RefCell::new(search_clients));

        clients.retain(|client| {
            for cmp_client in search_clients.borrow().iter() {
                if client.uid == cmp_client.uid {
                    search_clients
                        .borrow_mut()
                        .retain(|cmp_client_babe| cmp_client_babe.uid != cmp_client.uid);
                    return false;
                }
            }
            return true;
        });

        Ok(())
    }

    fn disconnect_all(&self) -> Result<(), Vec<Box<dyn std::error::Error>>> {
        let mut client = match self.registered_client.write() {
            Ok(v) => v,
            Err(_) => return Err(vec![QuickSocketError::ClientDataInvalid.to_box()]),
        };

        client.clear();
        client.shrink_to_fit();

        Ok(())
    }

    fn destroy_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut is_event_listener_on = match self.is_event_listener_on.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };
        *is_event_listener_on = false;
        let mut is_destroyed = match self.is_destroyed.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };
        *is_destroyed = true;
        Ok(())
    }

    fn register_event_handler(
        &self,
        event: String,
        func: fn(
            Arc<dyn ChannelImpl>,
            JsonValue,
            ChannelClient,
        ) -> Result<Option<JsonValue>, Box<QuickSocketError>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if match self.event_handlers.read() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        }
        .get(&event)
            != None
        {
            return Err(QuickSocketError::EventAlreadyExists.to_box());
        }
        match self.event_handlers.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        }
        .insert(event, func);
        Ok(())
    }

    fn register_client(&self, client: ChannelClient) -> Result<(), Box<dyn std::error::Error>> {
        let mut locked_client_list = match self.registered_client.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let cloned_client = client.clone();

        if locked_client_list
            .iter()
            .find(move |v| {
                if v.addr == cloned_client.addr || v.uid == cloned_client.uid {
                    return true;
                }
                false
            })
            .is_some()
        {
            return Err(QuickSocketError::ClientAlreadyExists.to_box());
        };

        locked_client_list.push(client);
        Ok(())
    }

    fn reconnect_client_by_uid(
        &self,
        uid: String,
        new_client: ChannelClient,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut locked_client_list = match self.registered_client.write() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let found_client = locked_client_list.iter_mut().find(move |v| v.uid == uid);
        if found_client.is_some() {
            let found_client = match found_client {
                Some(v) => v,
                None => return Err(QuickSocketError::ClientNotRegistered.to_box()),
            };
            std::mem::replace(found_client, new_client);
        };
        Ok(())
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

    pub fn create_tcp_channel(
        &self,
        setter: fn(&mut TcpChannel),
        pref: ChannelCreatePreferences,
    ) -> Result<Arc<TcpChannel>, Box<dyn std::error::Error>> {
        let addr = "127.0.0.1:0";

        let sock_ins = Arc::new(RwLock::from(TcpListener::bind(addr)?));

        let locked_tmp = sock_ins.clone();

        let locked_listener = match locked_tmp.read() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let port = locked_listener.local_addr()?.port();

        drop(locked_listener);

        let mut channel = TcpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: sock_ins,
            registered_client: Arc::new(RwLock::from(vec![])),
            port,
            event_handlers: Arc::new(RwLock::from(HashMap::new())),
            is_destroyed: Arc::new(RwLock::from(false)),
            is_event_listener_on: Arc::new(RwLock::from(true)),
            glob_instance: match self.self_instance.clone() {
                Some(v) => v,
                None => {
                    return Err(QuickSocketError::InstanceInitializeInvalid.to_box());
                }
            },
            pref: pref.clone(),
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

        drop(mutex);

        let channel_clone = channel.clone();

        if *channel.is_event_listener_on.read().unwrap() {
            thread::spawn(move || {
                log::debug!("TCP Thread spawned");
                for instance in channel.instance.read().unwrap().incoming() {
                    if *channel.is_destroyed.read().unwrap() {
                        break;
                    }
                    log::debug!("TCP For loop going");

                    let instance = match instance {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if pref.concurrent {
                        &instance.set_read_timeout(Some(Duration::from_millis(50)));
                    }

                    let addr = match instance.local_addr() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let ws = match accept(instance) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Error from handshake...{}", e);
                            continue;
                        }
                    };

                    let accepted_client = ChannelClient::new(addr, Some(ws));

                    let channel_closure_clone = channel.clone();

                    thread::spawn(move || {
                        let channel = channel_closure_clone;
                        loop {
                            let mut val = match accepted_client.stream.clone() {
                                Some(v) => v,
                                None => return,
                            };

                            let mut write_locked = match val.write() {
                                Ok(v) => v,
                                Err(_) => return,
                            };

                            let str_val = match write_locked.read_message() {
                                Ok(v_msg) => {
                                    drop(write_locked);
                                    match v_msg.into_text() {
                                        Ok(str_val) => str_val,
                                        Err(e) => {
                                            channel.emit_to(
                                                vec![accepted_client.clone()],
                                                ResponseEvent::Error,
                                                QuickSocketError::SocketBufferReadFail.jsonify(),
                                            );
                                            return;
                                        }
                                    }
                                }
                                Err(e) => {
                                    drop(write_locked);

                                    let res = match e {
                                        tungstenite::Error::Io(e_io) => {
                                            let v = e_io.kind() == ErrorKind::WouldBlock;
                                            // || e_io.kind() == ErrorKind::TimedOut;
                                            v
                                        }
                                        _ => false,
                                    };

                                    if res == true {
                                        thread::sleep(Duration::from_millis(10));
                                        continue;
                                    }

                                    channel.emit_to(
                                        vec![accepted_client.clone()],
                                        ResponseEvent::Error,
                                        QuickSocketError::SocketBufferReadFail.jsonify(),
                                    );
                                    return;
                                }
                            };

                            log::debug!("TCP data accepted: {}", &str_val);

                            let msg = match json::parse(&str_val) {
                                Ok(v) => v,
                                Err(_) => {
                                    channel.emit_to(
                                        vec![accepted_client.clone()],
                                        ResponseEvent::Error,
                                        QuickSocketError::JsonParseFail.jsonify(),
                                    );
                                    continue;
                                }
                            };

                            let event = &msg["event"];
                            if !event.is_string() {
                                channel.emit_to(
                                    vec![accepted_client.clone()],
                                    ResponseEvent::Error,
                                    QuickSocketError::JsonFormatInvalid.jsonify(),
                                );
                                continue;
                            }

                            let event_handler_locked = match channel.event_handlers.read() {
                                Ok(v) => v,
                                Err(_) => continue,
                            };

                            let event_handler = match event_handler_locked.get(&event.to_string()) {
                                Some(v) => v,
                                None => {
                                    channel.emit_to(
                                        vec![accepted_client.clone()],
                                        ResponseEvent::Error,
                                        QuickSocketError::EventNotFound.jsonify(),
                                    );
                                    continue;
                                }
                            };

                            match event_handler(
                                channel.clone(),
                                msg["data"].to_owned(),
                                accepted_client.clone(),
                            ) {
                                Ok(v) => {
                                    if let Some(value) = v {
                                        channel.emit_to(
                                            vec![accepted_client.clone()],
                                            ResponseEvent::Ok,
                                            value,
                                        );
                                    }
                                    continue;
                                }
                                Err(e) => {
                                    channel.emit_to(
                                        vec![accepted_client.clone()],
                                        ResponseEvent::Error,
                                        e.jsonify(),
                                    );
                                    continue;
                                }
                            }
                        }
                    });
                }
            });
        }

        println!("TCP Channel opened on port : {}", channel_clone.port);

        Ok(channel_clone)
    }

    pub fn create_udp_channel(
        &self,
        setter: fn(&mut UdpChannel),
        pref: ChannelCreatePreferences,
    ) -> Result<Arc<UdpChannel>, Box<dyn std::error::Error>> {
        let addr = "127.0.0.1:0";

        let sock_ins = Arc::new(RwLock::from(UdpSocket::bind(addr)?));

        let locked_temp = sock_ins.clone();

        let locked_listener = match locked_temp.read() {
            Ok(v) => v,
            Err(_) => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
        };

        let port = locked_listener.local_addr()?.port();

        drop(locked_listener);

        let mut channel = UdpChannel {
            channel_id: Uuid::new_v4().to_string(),
            instance: sock_ins,
            registered_client: Arc::new(RwLock::from(vec![])),
            port,
            event_handlers: Arc::new(RwLock::from(HashMap::new())),
            is_destroyed: Arc::new(RwLock::from(false)),
            is_event_listener_on: Arc::new(RwLock::from(true)),
            glob_instance: match self.self_instance.clone() {
                Some(v) => v,
                None => return Err(QuickSocketError::ChannelInitializeFail.to_box()),
            }
            .clone(),
            pref: pref.clone(),
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

        if *channel.is_event_listener_on.read().unwrap() {
            thread::spawn(move || {
                log::debug!("UDP Thread spawned!");
                while !*channel.is_destroyed.read().unwrap() {
                    let mut buf: [u8; 65535] = [0; 65535];
                    let received = || -> Result<_, Box<dyn std::error::Error>> {
                        Ok(channel.instance.read()?.recv_from(&mut buf)?)
                    }();
                    let (size, addr) = match received {
                        Ok(v) => v,
                        Err(_) => {
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
                                Err(_) => {
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

                            let event_handler_locked = match channel.event_handlers.read() {
                                Ok(v) => v,
                                Err(_) => {
                                    channel.emit_to(
                                        temp_client!(addr),
                                        ResponseEvent::Error,
                                        QuickSocketError::ChannelInitializeFail.jsonify(),
                                    );
                                    return;
                                }
                            };

                            let event_handler = match event_handler_locked.get(&event.to_string()) {
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

                            match event_handler(
                                channel.clone(),
                                msg["data"].to_owned(),
                                ChannelClient {
                                    addr: addr.clone(),
                                    stream: None,
                                    uid: String::new(),
                                },
                            ) {
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
}
