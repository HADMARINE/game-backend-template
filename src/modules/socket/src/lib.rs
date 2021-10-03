use neon::prelude::*;
mod error;
mod js_interface;
mod socket_instance;
mod util;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("hello node"))
}

fn create_room(mut cx: FunctionContext) -> JsResult<JsObject> {}
// fn close_room
// fn

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("hello", hello)?;
    Ok(())
}
