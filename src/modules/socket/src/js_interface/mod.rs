use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, net::SocketAddr, rc::Rc, sync::{Arc, RwLock}};

use json::JsonValue;
use neon::{prelude::{Context, Finalize, FunctionContext, Handle, JsArray, JsBoolean, JsBox, JsFunction, JsNull, JsNumber, JsObject, JsResult, JsString, JsUndefined, JsValue, Object, Value}, result::Throw};

use crate::{error::predeclared::QuickSocketError, socket_instance::ChannelImpl};

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

enum JsonTypes {
    Array,
    Boolean,
    Null,
    Number,
    Object,
    String,
    Unknown
}

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
            value.
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

        for key in data_keys {
            let value = data.get(&mut cx, key)?.is_a<(cx);
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

    pub fn determine_js_type(&self, value: &Handle<JsValue>) -> JsonTypes {
        if value.is_a::<JsArray, _>(self.cx.get_mut()) {
            JsonTypes::Array
        } else if value.is_a::<JsBoolean,_>(self.cx.get_mut()) {
            JsonTypes::Boolean
        }   else if value.is_a::<JsNull,_>(self.cx.get_mut()) {
            JsonTypes::Null
        }else if value.is_a::<JsNumber,_>(self.cx.get_mut()) {
            JsonTypes::Number
        }else if value.is_a::<JsObject,_>(self.cx.get_mut()) {
            JsonTypes::Object
        } else if value.is_a::<JsString,_>(self.cx.get_mut()) {
            JsonTypes::String
        }else {
            JsonTypes::Unknown
        }
    }

    pub fn determine_json_type(&self, value: JsonValue) -> JsonTypes {
        if value.is_array() {
            JsonTypes::Array
        } else if value.is_boolean() {
            JsonTypes::Boolean
        } else if value.is_null() {
            JsonTypes::Null
        } else if value.is_number() {
            JsonTypes::Number
        } else if value.is_object() {
            JsonTypes::Object
        } else if value.is_string() {
            JsonTypes::String
        } else {
            JsonTypes::Unknown
        }
    }

    pub fn parse_js_to_json(&self, value: Handle<JsObject>) -> Result<json::object::Object, Box<dyn std::error::Error>> {
        fn array<'a>(instance:&JsInterface<'a>, value: Handle<JsValue>) -> Result<json::JsonValue,Throw> {
            let values:Handle<JsArray> = value.downcast_or_throw(instance.cx.get_mut())?;
            let values = values.to_vec(instance.cx.get_mut())?;

            let mut return_value = json::JsonValue::new_array();
            
            for v in values {
                match instance.determine_js_type(&v) {
                    JsonTypes::Array => {
                         return_value.push(array(&instance, v)?);
                    }
                    JsonTypes::Boolean => {
                        return_value.push(boolean(&instance, v)?);
                    }
                    JsonTypes::Null => {
                        return_value.push(null(&instance, v)?);
                    }
                    JsonTypes::Number => {
                        return_value.push(number(&instance, v)?);
                    }
                    JsonTypes::Object => {
                        return_value.push(object(&instance, v)?);
                    }
                    JsonTypes::String => {
                        return_value.push(string(&instance, v)?);
                    }
                    JsonTypes::Unknown => {
                        return instance.cx.borrow().throw_error("json data invalid");
                    }
                }
            };

            Ok(return_value)
        }
        fn boolean<'a>(instance:&JsInterface<'a>, value: Handle<JsValue>) -> Result<json::JsonValue,Throw> { // ? check boolean type of json
            let value:Handle<JsBoolean> = value.downcast_or_throw(instance.cx.get_mut())?;
            let value = value.value(instance.cx.get_mut());

            Ok(value.into())
        }
        fn null<'a>(instance:&JsInterface<'a>, value: Handle<JsValue>) -> Result<json::JsonValue,Throw> { 
            Ok(JsonValue::Null)
        }
        fn number<'a>(instance:&JsInterface<'a>, value: Handle<JsValue>) -> Result<json::number::Number,Throw> { 
            let value:Handle<JsNumber> = value.downcast_or_throw(instance.cx.get_mut())?;
            let value= value.value(instance.cx.get_mut());

            Ok(value.into())
        }
        fn object<'a>(instance:&JsInterface<'a>, value: Handle<JsValue>) -> Result<json::object::Object,Throw> { 
            // determine type of value with determine_json_type fn
            let master_value:Handle<JsObject> = value.downcast_or_throw(instance.cx.get_mut())?;
            let keys = master_value.get_own_property_names(instance.cx.get_mut())?.to_vec(instance.cx.get_mut())?;

            for key in keys {
                let value = master_value.get(instance.cx.get_mut(), key.clone())?;
                match instance.determine_js_type(&value) {
                    // TODO : Here
                }
            }

            Ok(())
            
        }
        fn string<'a>(instance:&JsInterface<'a>, value: Handle<JsValue>) -> Result<json::JsonValue,Throw> { 
            let value : Handle<JsString> = value.downcast_or_throw(instance.cx.get_mut())?;
            let value = value.value(instance.cx.get_mut());

            Ok(value.into())
        }

    }

    pub fn parse_json_to_js(&self, value: json::object::Object) -> Result<Handle<JsObject>, Box<dyn std::error::Error>> {
        // fn array()
        // fn booelan()
        // fn null()
        // fn number()
        // fn object()
        // fn string()
    }
}

pub fn socket_data_handler(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let interface = cx.argument::<BoxedJsInterface>(0)?;
    interface.borrow_mut().socket_data_handler(cx)
}
