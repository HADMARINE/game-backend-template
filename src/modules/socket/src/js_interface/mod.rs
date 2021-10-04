use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use json::JsonValue;
use neon::prelude::{
    Context, FunctionContext, Handle, JsBoolean, JsObject, JsResult, JsString, JsValue, Object,
};

use crate::socket_instance::ChannelController;

pub struct JsInterface {
    js_handler: Option<fn(FunctionContext) -> JsResult<JsBoolean>>,
    event_list: HashMap<String, fn(JsonValue) -> Result<(), Box<dyn std::error::Error>>>,
    channel: ChannelController,
}

fn socket_data_handler(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let data: Handle<JsObject> = cx.argument(0)?;
    let event: Handle<JsValue> = data.get(&mut cx, "event")?;
    let data = data.get(&mut cx, "data")?;
    Ok(cx.boolean(true))
}

impl JsInterface {
    pub fn new() -> Self {
        JsInterface {
            js_handler: None,
            event_list: HashMap::new(),
        }
    }

    pub fn to_object<'a>(&self, &mut impl Context<'a>) -> 

    pub fn close_room(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    // Used by js
    pub fn socket_data_handler<'a>(&self, mut cx: FunctionContext<'a>) -> JsResult<JsBoolean> {
        let true_value = &mut cx.boolean(true);

        Ok(cx.boolean(true))
    }

    pub fn set_js_handler(
        &self,
        func: fn(FunctionContext) -> JsResult<JsBoolean>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.js_handler = Some(func);
        Ok(())
    }
}
