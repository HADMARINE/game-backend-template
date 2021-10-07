use std::{
    cell::RefCell,
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use json::JsonValue;
use neon::{
    prelude::{
        Context, Finalize, FunctionContext, Handle, JsBoolean, JsBox, JsFunction, JsObject,
        JsResult, JsUndefined, Object, Value,
    },
    result::Throw,
};

use crate::socket_instance::ChannelImpl;

type BoxedJsInterface = JsBox<RefCell<JsInterface>>;

pub struct JsInterface {
    js_handler: JsFunction,
    event_list:
        HashMap<String, Box<dyn FnOnce(JsonValue) -> Result<(), Box<dyn std::error::Error>>>>,
    pub addr: SocketAddr,
    channel: Arc<dyn ChannelImpl>,
}

impl Finalize for JsInterface {}
unsafe impl Sync for JsInterface {}
unsafe impl Send for JsInterface {}

impl JsInterface {
    pub fn new(
        js_handler: JsFunction,
        addr: SocketAddr,
        event_list: HashMap<
            String,
            Box<dyn FnOnce(JsonValue) -> Result<(), Box<dyn std::error::Error>>>,
        >,
        channel: Arc<dyn ChannelImpl>,
    ) -> Self {
        JsInterface {
            js_handler, // event handler function runs on js thread
            event_list, // event handler list runs on rust thread
            addr,
            channel,
        }
    }

    pub fn socket_data_handler(&self, mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let data: Handle<JsObject> = cx.argument(0)?;
        let event = data
            .get(&mut cx, "event")?
            .to_string(&mut cx)?
            .value(&mut cx);
        let data: Handle<JsObject> = data.get(&mut cx, "data")?.downcast_or_throw(&mut cx)?;

        let data_keys = data.get_own_property_names(&mut cx)?.to_vec(&mut cx)?;

        let mut mapped_data: JsonValue = JsonValue::new_object();

        for k in data_keys {
            // TODO : create parser here!
        }

        let handler = match self.event_list.get(&event.to_string()) {
            Some(v) => v,
            None => return Err(Throw),
        };

        let dd: Handle<JsObject> = match data.downcast(&mut cx) {
            Ok(v) => v,
            Err(e) => return cx.throw_error("value invalid"),
        };

        Ok(cx.undefined())
    }
}

pub fn socket_data_handler(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let interface = cx.argument::<BoxedJsInterface>(0)?;
    interface.borrow_mut().socket_data_handler(cx)
}
