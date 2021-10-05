use std::{collections::HashMap, net::SocketAddr};

use json::JsonValue;
use neon::{
    prelude::{
        FunctionContext, Handle, JsBoolean, JsFunction, JsObject, JsResult, JsUndefined, JsValue,
        Object,
    },
    result::Throw,
};

pub struct JsInterface {
    js_handler: JsFunction,
    event_list: HashMap<String, fn(JsonValue) -> Result<(), Box<dyn std::error::Error>>>,
    addr: SocketAddr,
}

impl JsInterface {
    pub fn new(handler: JsFunction, addr: SocketAddr) -> Self {
        JsInterface {
            js_handler: handler,
            event_list: HashMap::new(),
            addr,
        }
    }

    pub fn to_object(&self, &mut cx: &mut FunctionContext) -> JsResult<JsObject> {}

    pub fn close_room(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn socket_data_handler(&self, mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let data: Handle<JsObject> = cx.argument(0)?;
        let event = data.get(&mut cx, "event")?;
        let data = data.get(&mut cx, "data")?;

        let handler = match self.event_list.get(event) {
            Some(v) => v,
            None => return Err(Throw),
        };

        handler()
    }

    pub fn set_js_handler(
        &self,
        func: fn(FunctionContext) -> JsResult<JsBoolean>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.js_handler = Some(func);
        Ok(())
    }
}
