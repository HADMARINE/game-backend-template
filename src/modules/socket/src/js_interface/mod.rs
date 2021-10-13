use std::{
    cell::RefCell, collections::HashMap, convert::TryInto, net::SocketAddr, rc::Rc, sync::Arc,
};

use json::JsonValue;
use neon::{
    prelude::{
        Context, Finalize, FunctionContext, Handle, JsArray, JsBoolean, JsBox, JsFunction, JsNull,
        JsNumber, JsObject, JsResult, JsString, JsUndefined, JsValue, Object,
    },
    result::Throw,
};

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
    Unknown,
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

    pub fn to_js_box(cx: &mut FunctionContext, value: JsInterface) -> BoxedJsInterface {
        let owned_self = value;

        let res = JsBox::new(cx, RefCell::new(owned_self));
        *res
    }

    pub fn call_js_handler(
        &self,
        event: String,
        data: json::object::Object,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event: Handle<JsValue> = JsString::new(&mut *self.cx.borrow_mut(), event).upcast();
        let parsed_data: Handle<JsValue> = self.parse_json_to_js(data)?.upcast();

        self.js_handler.call(
            &mut *self.cx.borrow_mut(),
            self.cx.borrow_mut().null(),
            vec![event, parsed_data],
        );

        Ok(())
    }

    pub fn socket_data_handler<'b>(
        &self,
        mut cx: FunctionContext<'b>,
    ) -> JsResult<'b, JsUndefined> {
        let event: Handle<JsString> = cx.argument(0)?;
        let data: Handle<JsObject> = cx.argument(1)?;

        let __data = self.parse_js_to_json(data)?;

        let handler = match self.event_list.get(&event.value(&mut cx)) {
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
        if value.is_a::<JsArray, _>(&mut *self.cx.borrow_mut()) {
            JsonTypes::Array
        } else if value.is_a::<JsBoolean, _>(&mut *self.cx.borrow_mut()) {
            JsonTypes::Boolean
        } else if value.is_a::<JsNull, _>(&mut *self.cx.borrow_mut()) {
            JsonTypes::Null
        } else if value.is_a::<JsNumber, _>(&mut *self.cx.borrow_mut()) {
            JsonTypes::Number
        } else if value.is_a::<JsObject, _>(&mut *self.cx.borrow_mut()) {
            JsonTypes::Object
        } else if value.is_a::<JsString, _>(&mut *self.cx.borrow_mut()) {
            JsonTypes::String
        } else {
            JsonTypes::Unknown
        }
    }

    pub fn determine_json_type(&self, value: &JsonValue) -> JsonTypes {
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

    pub fn parse_js_to_json(&self, value: Handle<JsObject>) -> Result<json::object::Object, Throw> {
        fn array<'a>(
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::Array, Throw> {
            let values: Handle<JsArray> =
                value.downcast_or_throw(&mut *instance.cx.borrow_mut())?;
            let values = values.to_vec(&mut *instance.cx.borrow_mut())?;

            let mut return_value = vec![];

            for v in values {
                match instance.determine_js_type(&v) {
                    JsonTypes::Array => {
                        return_value.push(array(&instance, v)?.into());
                    }
                    JsonTypes::Boolean => {
                        return_value.push(boolean(&instance, v)?);
                    }
                    JsonTypes::Null => {
                        return_value.push(null(&instance, v)?);
                    }
                    JsonTypes::Number => {
                        return_value.push(number(&instance, v)?.into());
                    }
                    JsonTypes::Object => {
                        return_value.push(object(&instance, v)?.into());
                    }
                    JsonTypes::String => {
                        return_value.push(string(&instance, v)?);
                    }
                    JsonTypes::Unknown => {
                        return instance.cx.borrow_mut().throw_error("json data invalid");
                    }
                }
            }

            Ok(return_value)
        }
        fn boolean<'a>(
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::JsonValue, Throw> {
            // ? check boolean type of json
            let value: Handle<JsBoolean> =
                value.downcast_or_throw(&mut *instance.cx.borrow_mut())?;
            let value = value.value(&mut *instance.cx.borrow_mut());

            Ok(value.into())
        }
        fn null<'a>(
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::JsonValue, Throw> {
            Ok(JsonValue::Null)
        }
        fn number<'a>(
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::number::Number, Throw> {
            let value: Handle<JsNumber> =
                value.downcast_or_throw(&mut *instance.cx.borrow_mut())?;
            let value = value.value(&mut *instance.cx.borrow_mut());

            Ok(value.into())
        }
        fn object<'a>(
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::object::Object, Throw> {
            // determine type of value with determine_json_type fn
            let master_value: Handle<JsObject> =
                value.downcast_or_throw(&mut *instance.cx.borrow_mut())?;
            let keys = master_value
                .get_own_property_names(&mut *instance.cx.borrow_mut())?
                .to_vec(&mut *instance.cx.borrow_mut())?;

            let mut return_value = json::object::Object::new();

            for key in keys {
                let value = master_value.get(&mut *instance.cx.borrow_mut(), key.clone())?;
                let key: Handle<JsString> =
                    key.downcast_or_throw(&mut *instance.cx.borrow_mut())?;
                let key = key.value(&mut *instance.cx.borrow_mut());
                match instance.determine_js_type(&value) {
                    JsonTypes::Array => {
                        return_value.insert(key.as_str(), array(&instance, value)?.into());
                    }
                    JsonTypes::Boolean => {
                        return_value.insert(key.as_str(), boolean(&instance, value)?);
                    }
                    JsonTypes::Null => {
                        return_value.insert(key.as_str(), null(&instance, value)?);
                    }
                    JsonTypes::Number => {
                        return_value.insert(key.as_str(), number(&instance, value)?.into());
                    }
                    JsonTypes::Object => {
                        return_value.insert(key.as_str(), object(&instance, value)?.into());
                    }
                    JsonTypes::String => {
                        return_value.insert(key.as_str(), string(&instance, value)?);
                    }
                    JsonTypes::Unknown => {
                        return instance.cx.borrow_mut().throw_error("json parse fail");
                    }
                }
            }

            Ok(return_value)
        }
        fn string<'a>(
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::JsonValue, Throw> {
            let value: Handle<JsString> =
                value.downcast_or_throw(&mut *instance.cx.borrow_mut())?;
            let value = value.value(&mut *instance.cx.borrow_mut());

            Ok(value.into())
        }

        Ok(match object(&self, value.upcast()) {
            Ok(v) => v,
            Err(_) => return self.cx.borrow_mut().throw_error("json parse fail"),
        })
    }

    pub fn parse_json_to_js(
        &self,
        value: json::object::Object,
    ) -> Result<Handle<JsObject>, Box<dyn std::error::Error>> {
        fn array<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsArray>, Box<dyn std::error::Error>> {
            let mut return_array: Vec<Handle<JsValue>> = vec![];

            for v in value.members() {
                let val = match instance.determine_json_type(&v) {
                    JsonTypes::Array => array(&instance, v.to_owned())?.upcast(),
                    JsonTypes::Boolean => boolean(&instance, v.to_owned())?.upcast(),
                    JsonTypes::Null => null(&instance, v.to_owned())?.upcast(),
                    JsonTypes::Number => number(&instance, v.to_owned())?.upcast(),
                    JsonTypes::Object => object(&instance, v.to_owned())?.upcast(),
                    JsonTypes::String => string(&instance, v.to_owned())?.upcast(),
                    JsonTypes::Unknown => return Err(QuickSocketError::JsonParseFail.to_box()),
                };

                return_array.push(val);
            }

            let js_array = JsArray::new(
                &mut *instance.cx.borrow_mut(),
                return_array.len().try_into()?,
            );

            for (i, s) in return_array.iter().enumerate() {
                match js_array.set(&mut *instance.cx.borrow_mut(), i as u32, s.to_owned()) {
                    Ok(v) => continue,
                    Err(_) => return Err(QuickSocketError::JsonParseFail.to_box()),
                };
            }

            Ok(js_array)
        }
        fn boolean<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsBoolean>, Box<dyn std::error::Error>> {
            let value = match value.as_bool() {
                Some(v) => v,
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsBoolean::new(&mut *instance.cx.borrow_mut(), value))
        }
        fn null<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsNull>, Box<dyn std::error::Error>> {
            Ok(JsNull::new(&mut *instance.cx.borrow_mut()))
        }
        fn number<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsNumber>, Box<dyn std::error::Error>> {
            let v = match value.as_f64() {
                Some(v) => v,
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsNumber::new::<_, f64>(&mut *instance.cx.borrow_mut(), v))
        }
        fn object<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsObject>, Box<dyn std::error::Error>> {
            let jsObject = JsObject::new(&mut *instance.cx.borrow_mut());

            for (key, value) in value.entries() {
                let value = value.to_owned();
                let value: Handle<JsValue> = match instance.determine_json_type(&value) {
                    JsonTypes::Array => array(&instance, value)?.upcast(),
                    JsonTypes::Boolean => boolean(&instance, value)?.upcast(),
                    JsonTypes::Null => null(&instance, value)?.upcast(),
                    JsonTypes::Number => number(&instance, value)?.upcast(),
                    JsonTypes::Object => object(&instance, value)?.upcast(),
                    JsonTypes::String => string(&instance, value)?.upcast(),
                    JsonTypes::Unknown => return Err(QuickSocketError::JsonParseFail.to_box()),
                };

                jsObject.set(&mut *instance.cx.borrow_mut(), key, value);
            }

            Ok(jsObject)
        }
        fn string<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsString>, Box<dyn std::error::Error>> {
            let value = match value.as_str() {
                Some(v) => v.to_string(),
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsString::new(&mut *instance.cx.borrow_mut(), value))
        }

        Ok(object(self, json::JsonValue::Object(value))?)
    }
}

pub fn socket_data_handler(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let interface = cx.argument::<BoxedJsInterface>(0)?;
    let borrowed = interface.borrow_mut();
    borrowed.socket_data_handler(cx)
}
