use crate::js_interface::JsInterface;

use json::JsonValue;
use neon::prelude::{
    Context, FunctionContext, Handle, JsObject, JsResult, JsString, JsUndefined, JsValue,
};
use std::collections::HashMap;

// pub mod echo;

pub type BridgeMapType = HashMap<String, BridgeHandlerType>;

pub type BridgeHandlerType =
    Box<dyn Fn(JsonValue, &JsInterface) -> Result<(), Box<dyn std::error::Error>>>;

pub fn manager() -> BridgeMapType {
    let mut map: BridgeMapType = HashMap::new();
    map.insert(String::from("print"), Box::new(print));
    map
}

pub fn resolver<'a>(
    cx: &'a mut FunctionContext<'a>,
    event: Handle<'a, JsString>,
    data: Handle<'a, JsObject>,
) -> JsResult<'a, JsUndefined> {
    let manager_data = manager();
    let v = match manager_data.get(&event.value(cx)) {
        Some(v) => v,
        None => return cx.throw_error("invalid event name"),
    };

    let data = match json::parse(data.as_str()) {
        Ok(v) => v,
        Err(_) => return cx.throw_error("json parse failed"),
    };

    match v(data) {
        Ok(v) => (),
        Err(_) => return cx.throw_error("event handler has failed to resolve"),
    };

    Ok(cx.undefined())
}

fn print(value: JsonValue, interface: &JsInterface) -> Result<(), Box<dyn std::error::Error>> {
    println!("value: {}", value.to_string());
    Ok(())
}
