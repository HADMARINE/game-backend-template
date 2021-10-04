use std::collections::HashMap;

use neon::prelude::{FunctionContext, Handle, JsBoolean, JsObject, JsResult, Object};

pub struct JsInterface {
    js_handler: Option<fn(FunctionContext) -> JsResult<JsBoolean>>,
    event_list: HashMap<String, fn(JsonValue) -> Result<(), Box<dyn std::error::Error>>,
}

impl JsInterface {
    pub fn new() -> Self {
        JsInterface { 
            js_handler: None, 
            event_list: HashMap::new(),
        }
    }

    pub fn close_room(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn socket_data_handler(
        &self,
        mut cx: FunctionContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data: Handle<JsObject> = cx.argument(0)?;
        let event = data.get(&mut cx, "event");
        let data = data.get(&mut cx, "data");
        Ok(())
    }

    pub fn set_js_handler(
        &self,
        func: fn(FunctionContext) -> JsResult<JsBoolean>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.js_handler = Some(func);
        Ok(())
    }
}
