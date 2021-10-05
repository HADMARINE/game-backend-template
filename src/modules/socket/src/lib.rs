use std::sync::{Arc, RwLock};

use neon::{prelude::*, result::Throw};
mod error;
mod js_interface;
mod socket_instance;
mod util;

use socket_instance::{QuickSocketInstance, TcpChannelCreatePreferences};

static INSTANCE: Arc<RwLock<QuickSocketInstance>> = QuickSocketInstance::new();

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

fn create_tcp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let preferences = match TcpChannelCreatePreferences::from_jsobj(&mut cx, cx.argument(0)?){
        Ok(v) => v,
        Err(e) => return Err(Throw),
    }; // Preferences

    let write_locked = INSTANCE.write()?;
    let channel = write_locked.create_tcp_channel(|_| {}, preferences)?;
    drop(write_locked);
    let interface = js_interface::JsInterface::new().to_object(&mut cx)?;
}

fn create_udp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
    let preferences
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    let instance = QuickSocketInstance::new();
    cx.export_function("createTcpChannel", create_tcp_channel)?;
    cx.export_function("createUdpChannel", create_udp_channel)?;
    Ok(())
}
