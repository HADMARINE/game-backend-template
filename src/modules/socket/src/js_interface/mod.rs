use std::{
    cell::RefCell,
    collections::HashMap,
    net::SocketAddr,
    rc::Rc,
    sync::{Arc, RwLock},
};

use json::JsonValue;
use neon::{
    prelude::{
        Context, Finalize, FunctionContext, Handle, JsBoolean, JsBox, JsFunction, JsObject,
        JsResult, JsString, JsUndefined, Object, Value,
    },
    result::Throw,
};

use crate::socket_instance::ChannelImpl;

type BoxedJsInterface<'a> = JsBox<RefCell<JsInterface<'a>>>;

pub struct JsInterface<'a> {
    js_handler: JsFunction,
    event_list:
        HashMap<String, Box<dyn FnOnce(JsonValue) -> Result<(), Box<dyn std::error::Error>>>>,
    pub addr: SocketAddr,
    channel: Arc<dyn ChannelImpl>,
    pub cx: Rc<RefCell<FunctionContext<'a>>>,
}

impl<'a> Finalize for JsInterface<'a> {}
unsafe impl<'a> Sync for JsInterface<'a> {}
unsafe impl<'a> Send for JsInterface<'a> {}

impl<'a> JsInterface<'a> {
    pub fn new(
        js_handler: JsFunction,
        addr: SocketAddr,
        event_list: HashMap<
            String,
            Box<dyn FnOnce(JsonValue) -> Result<(), Box<dyn std::error::Error>>>,
        >,
        channel: Arc<dyn ChannelImpl>,
        cx: Rc<RefCell<FunctionContext<'a>>>,
    ) -> Self {
        JsInterface {
            js_handler, // event handler function runs on js thread
            event_list, // event handler list runs on rust thread
            addr,
            channel,
            cx,
        }
    }

    pub fn call_js_handler(
        &self,
        event: String,
        data: json::object::Object,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event = JsString::new(self.cx.get_mut(), event);
        let mapped_data = self.cx.borrow().empty_object();
        for (key, value) in data.iter() {
            // TODO : Create parser!
        }

        let final_data = self.cx.borrow().empty_object();
        final_data.set(self.cx.get_mut(), "event", event);
        final_data.set(self.cx.get_mut(), "data", mapped_data);

        self.js_handler.call(
            self.cx.get_mut(),
            self.cx.get_mut().null(),
            vec![final_data],
        );

        Ok(())
    }

    pub fn socket_data_handler<'b>(
        &self,
        mut cx: FunctionContext<'b>,
    ) -> JsResult<'b, JsUndefined> {
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
