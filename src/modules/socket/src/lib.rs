use std::sync::{Arc, RwLock};

use neon::prelude::*;
mod error;
mod js_interface;
mod socket_instance;
mod util;

use socket_instance::QuickSocketInstance;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

// fn create_tcp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {
//     let channel =
//     let interface = js_interface::JsInterface::new().to_object(&mut cx)?;
// }
fn create_tcp_channel(
    instance: Arc<RwLock<QuickSocketInstance>>,
) -> impl FnOnce(FunctionContext) -> JsResult<JsObject> {
    move |mut cx: FunctionContext| -> JsResult<JsObject> {
        let channel = instance.write().unwrap();
        let interface = js_interface::JsInterface::new().to_object(&mut cx)?;
        Ok(())
    }
}

fn create_udp_channel(mut cx: FunctionContext) -> JsResult<JsObject> {}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    let instance = QuickSocketInstance::new();
    cx.export_function("hello", hello)?;
    Ok(())
}
