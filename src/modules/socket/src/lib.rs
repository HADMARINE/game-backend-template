use std::{
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

static INSTANCE: Arc<RwLock<QuickSocketInstance>> = QuickSocketInstance::new();

fn create_tcp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let preferences = match TcpChannelCreatePreferences::from_jsobj(&mut cx, cx.argument(0)?) {
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

    let rc_cx = Rc::new(RefCell::from(cx));

    let interface = js_interface::JsInterface::new(
        *handler,
        match match channel.instance.read() {
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
        rc_cx,
    );

    let mut return_object = interface.cx.borrow().empty_object();
    return_object.set(
        interface.cx.get_mut(),
        "port",
        interface.cx.borrow().number(interface.addr.port()),
    );
    return_object.set(
        interface.cx.get_mut(),
        "socket_handler",
        JsFunction::new(interface.cx.get_mut(), js_interface::socket_data_handler)?,
    );

    Ok(return_object)
}

fn create_udp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let preferences = match UdpChannelCreatePreferences::from_jsobj(&mut cx, cx.argument(0)?) {
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

    let rc_cx = Rc::new(RefCell::from(cx));

    let interface = js_interface::JsInterface::new(
        *handler,
        match match channel.instance.read() {
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
        rc_cx,
    );

    let mut return_object = interface.cx.borrow().empty_object();
    return_object.set(
        interface.cx.get_mut(),
        "port",
        interface.cx.borrow().number(interface.addr.port()),
    );
    return_object.set(
        interface.cx.get_mut(),
        "socketHandler",
        JsFunction::new(interface.cx.get_mut(), js_interface::socket_data_handler)?,
    );

    Ok(return_object)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createTcpChannel", create_tcp_channel)?;
    cx.export_function("createUdpChannel", create_udp_channel)?;
    Ok(())
}
