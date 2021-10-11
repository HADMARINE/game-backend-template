use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use neon::{prelude::*, result::Throw};
mod error;
mod js_interface;
mod socket_instance;
mod util;

use socket_instance::{
    QuickSocketInstance, TcpChannelCreatePreferences, UdpChannelCreatePreferences,
};

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
        HashMap::new(),
        channel,
        Rc::new(RefCell::from(cx)),
    );

    let mut cx = match interface.cx.try_borrow_mut() {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };

    let return_object = cx.empty_object();

    let port_value = cx.number(interface.addr.port());
    return_object.set(&mut *cx, "port", port_value)?;

    let socket_handler_value = JsFunction::new(&mut *cx, js_interface::socket_data_handler)?;
    return_object.set(&mut *cx, "socket_handler", socket_handler_value)?;

    Ok(return_object)
}

fn create_udp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let arg0 = cx.argument(0)?;
    let preferences = match UdpChannelCreatePreferences::from_jsobj(&mut cx, arg0) {
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
        HashMap::new(),
        channel,
        Rc::new(RefCell::from(cx)),
    );

    let mut cx = match interface.cx.try_borrow_mut() {
        Ok(v) => v,
        Err(_) => return Err(Throw),
    };

    let return_object = cx.empty_object();

    let port_value = cx.number(interface.addr.port());
    return_object.set(&mut *cx, "port", port_value)?;

    let socket_handler_value = JsFunction::new(&mut *cx, js_interface::socket_data_handler)?;
    return_object.set(&mut *cx, "socket_handler", socket_handler_value)?;

    Ok(return_object)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createTcpChannel", create_tcp_channel)?;
    cx.export_function("createUdpChannel", create_udp_channel)?;
    Ok(())
}
