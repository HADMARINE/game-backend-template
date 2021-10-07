use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use neon::{prelude::*, result::Throw};
mod error;
mod js_interface;
mod socket_instance;
mod util;

use socket_instance::{QuickSocketInstance, TcpChannelCreatePreferences};

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

    let interface = js_interface::JsInterface::new(
        *handler,
        match match channel.instance.read() {
            Ok(v) => v,
            Err(_) => return Err(cx.throw_error("instance initialize invalid")?),
        }
        .local_addr()
        {
            Ok(v) => v,
            Err(_) => return Err(cx.throw_error("instance initialize invalid")?),
        },
        HashMap::new(),
        channel,
    );

    // let boxed_interface = cx.boxed(interface);

    let mut return_object = cx.empty_object();
    return_object.set(&mut cx, "port", cx.number(interface.addr.port()));
    return_object.set(
        &mut cx,
        "socket_handler",
        JsFunction::new(&mut cx, js_interface::socket_data_handler)?,
    );

    Ok(return_object)
}

fn create_udp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    // let preferences
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    let instance = QuickSocketInstance::new();
    cx.export_function("createVacantTcpChannel", create_tcp_channel)?;
    cx.export_function("createUdpChannel", create_udp_channel)?;
    Ok(())
}
