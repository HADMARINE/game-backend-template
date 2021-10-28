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
    let preferences = match TcpChannelCreatePreferences::from_jsobj(&mut cx, arg0) {
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

    let return_object = cx.empty_object();

    let port_value = cx.number(channel.port);
    return_object.set(&mut cx, "port", port_value)?;

    let uuid_value = cx.string(channel.channel_id.clone());
    return_object.set(&mut cx, "uuid", uuid_value);

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

    let return_object = cx.empty_object();

    let port_value = cx.number(channel.port);
    return_object.set(&mut cx, "port", port_value)?;

    let uuid_value = cx.string(channel.channel_id.clone());
    return_object.set(&mut cx, "uuid", uuid_value);

    Ok(return_object)
}

fn event_handler_rs(mut cx: FunctionContext) -> JsResult<JsUndefined> {}

fn set_js_event_handler(mut cx: FunctionContext) -> JsResult<JsUndefined> {}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createTcpChannel", create_tcp_channel)?;
    cx.export_function("createUdpChannel", create_udp_channel)?;
    cx.export_function("eventHandler", event_handler_rs)?;

    Ok(())
}
