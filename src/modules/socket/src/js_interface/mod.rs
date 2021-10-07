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
        JsResult, JsUndefined, Object,
    },
    result::Throw,
};

use crate::socket_instance::ChannelImpl;

type BoxedJsInterface = JsBox<RefCell<JsInterface>>;

pub struct JsInterface {
    js_handler: JsFunction,
    event_list: HashMap<String, JsFunction>,
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
        event_list: HashMap<String, JsFunction>,
        channel: Arc<dyn ChannelImpl>,
    ) -> Self {
        JsInterface {
            js_handler, // event handler in js instance
            event_list,
            addr,
            channel,
        }
    }

    // pub fn to_js_object<'a>(&self, &mut cx: &'a mut FunctionContext) -> JsResult<'a, JsObject> {
    //     let boxed_self = cx.boxed(*self);

    //     let obj = cx.empty_object();
    //     obj.set(&mut cx, "port", cx.number(self.addr.port()));
    //     obj.set(
    //         &mut cx,
    //         "eventHandler",
    //         JsFunction::new(&mut cx, boxed_self.socket_data_handler)?,
    //     );
    //     Ok(obj)
    // }

    pub fn socket_data_handler(&self, mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let data: Handle<JsObject> = cx.argument(0)?;
        let event = data.get(&mut cx, "event")?;
        let data = data.get(&mut cx, "data")?;

        let handler = match self.event_list.get(event) {
            Some(v) => v,
            None => return Err(Throw),
        };

        let dd: Handle<JsObject> = match data.downcast(&mut cx) {
            Ok(v) => v,
            Err(e) => return cx.throw_error("value invalid"),
        };

        handler.call(cx);
    }
}

pub fn socket_data_handler(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let interface = cx.argument::<BoxedJsInterface>(0)?;
    interface.borrow_mut().socket_data_handler(cx)
}
