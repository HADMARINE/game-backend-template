use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use json::object;
use neon::{prelude::*, result::Throw};
mod app;
mod error;
mod js_interface;
mod socket_instance;
mod util;

use socket_instance::{
    QuickSocketInstance, TcpChannelCreatePreferences, UdpChannelCreatePreferences,
};

use crate::error::predeclared::QuickSocketError;

lazy_static::lazy_static! {
    static ref INSTANCE: Arc<RwLock<QuickSocketInstance>> = QuickSocketInstance::new();
}

fn create_tcp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let arg0 = cx.argument(0)?;
    let mut preferences = match TcpChannelCreatePreferences::from_jsobj(&mut cx, arg0) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    }; // Preferences

    let handler: Handle<JsFunction> = cx.argument(1)?;

    let write_locked = match INSTANCE.write() {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };
    let channel = match write_locked.create_tcp_channel(|_| {}, preferences) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };

    drop(write_locked);

    let interface = js_interface::JsInterface::new(
        *handler,
        match match channel.clone().instance.read() {
            Ok(v) => v,
            Err(_) => return Err(cx.throw_error("instance init invalid")?),
        }
        .local_addr()
        {
            Ok(v) => v,
            Err(_) => return Err(cx.throw_error("instance init invalid")?),
        },
        channel,
        Arc::new(RwLock::from(cx)),
    );

    let cx_some = match interface.cx.clone() {
        Some(v) => v,
        None => panic!(QuickSocketError::InstanceInitializeInvalid),
    };
    let cx = &mut *match cx_some.try_write() {
        Ok(v) => v,
        Err(_) => panic!(QuickSocketError::InstanceInitializeInvalid),
    };

    let return_object = cx.empty_object();

    let port_value = cx.number(interface.addr.port());
    return_object.set(cx, "port", port_value)?;

    let socket_handler_value = JsFunction::new(cx, js_interface::socket_data_handler)?;
    return_object.set(cx, "socket_handler", socket_handler_value)?;

    let boxed_interface = js_interface::JsInterface::to_js_box(cx, interface);
    return_object.set(cx, "interface", boxed_interface)?;

    Ok(return_object)
}

fn create_udp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let arg0 = cx.argument(0)?;
    let mut preferences = match UdpChannelCreatePreferences::from_jsobj(&mut cx, arg0) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    }; // Preferences

    let handler: Handle<JsFunction> = cx.argument(1)?;

    let write_locked = match INSTANCE.write() {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };
    let channel = match write_locked.create_udp_channel(|_| {}, preferences) {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };

    drop(write_locked);

    let interface = js_interface::JsInterface::new(
        *handler,
        match match channel.clone().instance.read() {
            Ok(v) => v,
            Err(_) => return Err(cx.throw_error("instance init invalid")?),
        }
        .local_addr()
        {
            Ok(v) => v,
            Err(_) => return Err(cx.throw_error("instance init invalid")?),
        },
        channel,
        Arc::new(RwLock::from(cx)),
    );

    let cx_some = match interface.cx.clone() {
        Some(v) => v,
        None => panic!(QuickSocketError::InstanceInitializeInvalid),
    };
    let cx = &mut *match cx_some.try_write() {
        Ok(v) => v,
        Err(_) => panic!(QuickSocketError::InstanceInitializeInvalid),
    };

    let return_object = cx.empty_object();

    let port_value = cx.number(interface.addr.port());
    return_object.set(cx, "port", port_value)?;

    let socket_handler_value = JsFunction::new(cx, js_interface::socket_data_handler)?;
    return_object.set(cx, "socket_handler", socket_handler_value)?;

    let boxed_interface = js_interface::JsInterface::to_js_box(cx, interface);
    return_object.set(cx, "interface", boxed_interface)?;

    Ok(return_object)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createTcpChannel", create_tcp_channel)?;
    cx.export_function("createUdpChannel", create_udp_channel)?;
    Ok(())
}
