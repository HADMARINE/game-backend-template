use std::{
    borrow::BorrowMut,
    cell::RefCell,
    convert::TryInto,
    net::SocketAddr,
    rc::Rc,
    sync::{Arc, RwLock, RwLockWriteGuard},
};

use json::JsonValue;
use neon::{
    prelude::{
        CallContext, Context, Finalize, FunctionContext, Handle, JsArray, JsBoolean, JsBox,
        JsFunction, JsNull, JsNumber, JsObject, JsResult, JsString, JsUndefined, JsValue, Object,
    },
    result::Throw,
};

use crate::{
    app::bridge::{manager, BridgeMapType},
    error::predeclared::QuickSocketError,
    socket_instance::ChannelImpl,
};

type BoxedJsInterface<'a> = JsBox<RefCell<JsInterface<'a>>>;

#[macro_export]
macro_rules! get_cx {
    ($x:expr) => {{
        &mut *match $x {
            Some(v) => match v.try_write() {
                Ok(v) => v,
                Err(_) => panic!("try write error"),
            },
            None => panic!("none error"),
        }
    }};
}

macro_rules! get_cx_without_option {
    ($x: expr) => {{
        &mut *match $x.try_write() {
            Ok(v) => v,
            Err(_) => panic!("try write error"),
        }
    }};
}

pub struct JsHandlerContainer<'a> {
    handler: Arc<RwLock<Handle<'a, JsFunction>>>,
    cx: Arc<RwLock<FunctionContext<'a>>>,
}

impl<'a> JsHandlerContainer<'a> {
    fn new(cx: FunctionContext<'a>, handler: Handle<'a, JsFunction>) -> JsHandlerContainer<'a> {
        JsHandlerContainer {
            cx: Arc::new(RwLock::from(cx)),
            handler: Arc::new(RwLock::From(handler)),
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

    pub fn parse_json_to_js(
        &self,
        value: json::JsonValue,
    ) -> Result<Handle<JsObject>, Box<dyn std::error::Error>> {
        fn array<'a>(
            instance: &JsHandlerContainer<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsArray>, Box<dyn std::error::Error>> {
            let cx = get_cx_without_option!(instance.cx);

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

            let js_array = JsArray::new(cx, return_array.len().try_into()?);

            for (i, s) in return_array.iter().enumerate() {
                match js_array.set(cx, i as u32, s.to_owned()) {
                    Ok(_) => continue,
                    Err(_) => return Err(QuickSocketError::JsonParseFail.to_box()),
                };
            }

            Ok(js_array)
        }
        fn boolean<'a>(
            instance: &JsHandlerContainer<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsBoolean>, Box<dyn std::error::Error>> {
            let cx = get_cx_without_option!(instance.cx);

            let value = match value.as_bool() {
                Some(v) => v,
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsBoolean::new(cx, value))
        }
        fn null<'a>(
            instance: &JsHandlerContainer<'a>,
            _: JsonValue,
        ) -> Result<Handle<'a, JsNull>, Box<dyn std::error::Error>> {
            let cx = get_cx_without_option!(instance.cx);

            Ok(JsNull::new(cx))
        }
        fn number<'a>(
            instance: &JsHandlerContainer<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsNumber>, Box<dyn std::error::Error>> {
            let cx = get_cx_without_option!(instance.cx);

            let v = match value.as_f64() {
                Some(v) => v,
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsNumber::new::<_, f64>(cx, v))
        }
        fn object<'a>(
            instance: &JsHandlerContainer<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsObject>, Box<dyn std::error::Error>> {
            let cx = get_cx_without_option!(instance.cx);

            let jsObject = JsObject::new(cx);

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

                jsObject.set(cx, key, value);
            }

            Ok(jsObject)
        }
        fn string<'a>(
            instance: &JsHandlerContainer<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsString>, Box<dyn std::error::Error>> {
            let cx = get_cx_without_option!(instance.cx);

            let value = match value.as_str() {
                Some(v) => v.to_string(),
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsString::new(cx, value))
        }

        Ok(object(self, value)?)
    }

    fn call(&self, event: String, value: JsonValue) {
        let cx = get_cx_without_option!(self.cx);

        let null_value = cx.null();
        let event: Handle<JsValue> = JsString::new(cx, event).upcast();

        let js_value = match self.parse_json_to_js(value) {
            Ok(v) => v,
            Err(_) => panic!("error"),
        }
        .upcast();
        match self.handler.read() {
            Ok(v) => v,
            Err(_) => panic!("rwlock error"),
        }
        .call(cx, null_value, vec![event, js_value]);
    }
}

pub struct JsInterface<'a> {
    event_list: BridgeMapType,
    pub addr: SocketAddr,
    channel: Arc<dyn ChannelImpl>,
    pub cx: Option<Arc<RwLock<FunctionContext<'a>>>>,
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
        addr: SocketAddr,
        channel: Arc<dyn ChannelImpl>,
        cx: Option<Arc<RwLock<FunctionContext<'a>>>>,
    ) -> Self {
        JsInterface {
            event_list: manager(), // event handler list runs on rust thread
            addr,
            channel,
            cx,
        }
    }

    pub fn to_js_box<'b, 'c>(
        cx: &mut FunctionContext<'c>,
        value: JsInterface,
    ) -> Handle<'c, BoxedJsInterface<'b>> {
        let owned_self = value;

        let transfered_self = JsInterface::<'static> {
            addr: owned_self.addr,
            channel: owned_self.channel,
            event_list: owned_self.event_list,
            cx: None,
        };

        let res = JsBox::new(cx, RefCell::new(transfered_self));
        res
    }

    pub fn socket_data_handler<'b>(
        &self,
        mut cx: FunctionContext<'b>,
    ) -> JsResult<'b, JsUndefined> {
        let event: Handle<JsString> = cx.argument(1)?;
        let data: Handle<JsObject> = cx.argument(2)?;

        let __data = self.parse_js_to_json(&mut cx, data)?;

        let handler = match self.event_list.get(&event.value(&mut cx)) {
            Some(v) => v,
            None => return Err(Throw),
        };

        let json_value = self.parse_js_to_json(&mut cx, data)?;

        match handler(json_value.into(), self) {
            Err(_) => return cx.throw_error("handler returned error"),
            _ => (),
        };

        Ok(cx.undefined())
    }

    pub fn determine_js_type(
        &self,
        cx: &mut FunctionContext,
        value: &Handle<JsValue>,
    ) -> JsonTypes {
        if value.is_a::<JsArray, _>(cx) {
            JsonTypes::Array
        } else if value.is_a::<JsBoolean, _>(cx) {
            JsonTypes::Boolean
        } else if value.is_a::<JsNull, _>(cx) {
            JsonTypes::Null
        } else if value.is_a::<JsNumber, _>(cx) {
            JsonTypes::Number
        } else if value.is_a::<JsObject, _>(cx) {
            JsonTypes::Object
        } else if value.is_a::<JsString, _>(cx) {
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

    pub fn parse_js_to_json(
        &self,
        cx: &mut FunctionContext,
        value: Handle<JsObject>,
    ) -> Result<json::object::Object, Throw> {
        fn array<'a>(
            cx: &mut FunctionContext,
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::Array, Throw> {
            let values: Handle<JsArray> = value.downcast_or_throw(cx)?;
            let values = values.to_vec(cx)?;

            let mut return_value = vec![];

            for v in values {
                match instance.determine_js_type(cx, &v) {
                    JsonTypes::Array => {
                        return_value.push(array(cx, &instance, v)?.into());
                    }
                    JsonTypes::Boolean => {
                        return_value.push(boolean(cx, &instance, v)?);
                    }
                    JsonTypes::Null => {
                        return_value.push(null(cx, &instance, v)?);
                    }
                    JsonTypes::Number => {
                        return_value.push(number(cx, &instance, v)?.into());
                    }
                    JsonTypes::Object => {
                        return_value.push(object(cx, &instance, v)?.into());
                    }
                    JsonTypes::String => {
                        return_value.push(string(cx, &instance, v)?);
                    }
                    JsonTypes::Unknown => {
                        return cx.throw_error("json data invalid");
                    }
                }
            }

            Ok(return_value)
        }
        fn boolean<'a>(
            cx: &mut FunctionContext,
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::JsonValue, Throw> {
            let value: Handle<JsBoolean> = value.downcast_or_throw(cx)?;
            let value = value.value(cx);

            Ok(value.into())
        }
        fn null<'a>(
            cx: &mut FunctionContext,
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::JsonValue, Throw> {
            Ok(JsonValue::Null)
        }
        fn number<'a>(
            cx: &mut FunctionContext,
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::number::Number, Throw> {
            let value: Handle<JsNumber> = value.downcast_or_throw(cx)?;
            let value = value.value(cx);

            Ok(value.into())
        }
        fn object<'a>(
            cx: &mut FunctionContext,
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::object::Object, Throw> {
            let master_value: Handle<JsObject> = value.downcast_or_throw(cx)?;
            let keys = master_value.get_own_property_names(cx)?.to_vec(cx)?;

            let mut return_value = json::object::Object::new();

            for key in keys {
                let value = master_value.get(cx, key.clone())?;
                let key: Handle<JsString> = key.downcast_or_throw(cx)?;
                let key = key.value(cx);
                match instance.determine_js_type(cx, &value) {
                    JsonTypes::Array => {
                        return_value.insert(key.as_str(), array(cx, &instance, value)?.into());
                    }
                    JsonTypes::Boolean => {
                        return_value.insert(key.as_str(), boolean(cx, &instance, value)?);
                    }
                    JsonTypes::Null => {
                        return_value.insert(key.as_str(), null(cx, &instance, value)?);
                    }
                    JsonTypes::Number => {
                        return_value.insert(key.as_str(), number(cx, &instance, value)?.into());
                    }
                    JsonTypes::Object => {
                        return_value.insert(key.as_str(), object(cx, &instance, value)?.into());
                    }
                    JsonTypes::String => {
                        return_value.insert(key.as_str(), string(cx, &instance, value)?);
                    }
                    JsonTypes::Unknown => {
                        return cx.throw_error("json parse fail");
                    }
                }
            }

            Ok(return_value)
        }
        fn string<'a>(
            cx: &mut FunctionContext,
            instance: &JsInterface<'a>,
            value: Handle<JsValue>,
        ) -> Result<json::JsonValue, Throw> {
            let value: Handle<JsString> = value.downcast_or_throw(cx)?;
            let value = value.value(cx);

            Ok(value.into())
        }

        Ok(match object(cx, &self, value.upcast()) {
            Ok(v) => v,
            Err(_) => return cx.throw_error("json parse fail"),
        })
    }

    pub fn parse_json_to_js(
        &self,
        value: json::JsonValue,
    ) -> Result<Handle<JsObject>, Box<dyn std::error::Error>> {
        fn array<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsArray>, Box<dyn std::error::Error>> {
            let cx = get_cx!(&instance.cx);

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

            let js_array = JsArray::new(cx, return_array.len().try_into()?);

            for (i, s) in return_array.iter().enumerate() {
                match js_array.set(cx, i as u32, s.to_owned()) {
                    Ok(_) => continue,
                    Err(_) => return Err(QuickSocketError::JsonParseFail.to_box()),
                };
            }

            Ok(js_array)
        }
        fn boolean<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsBoolean>, Box<dyn std::error::Error>> {
            let cx = get_cx!(&instance.cx);

            let value = match value.as_bool() {
                Some(v) => v,
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsBoolean::new(cx, value))
        }
        fn null<'a>(
            instance: &JsInterface<'a>,
            _: JsonValue,
        ) -> Result<Handle<'a, JsNull>, Box<dyn std::error::Error>> {
            let cx = get_cx!(&instance.cx);

            Ok(JsNull::new(cx))
        }
        fn number<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsNumber>, Box<dyn std::error::Error>> {
            let cx = get_cx!(&instance.cx);

            let v = match value.as_f64() {
                Some(v) => v,
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsNumber::new::<_, f64>(cx, v))
        }
        fn object<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsObject>, Box<dyn std::error::Error>> {
            let cx = get_cx!(&instance.cx);

            let jsObject = JsObject::new(cx);

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

                jsObject.set(cx, key, value);
            }

            Ok(jsObject)
        }
        fn string<'a>(
            instance: &JsInterface<'a>,
            value: JsonValue,
        ) -> Result<Handle<'a, JsString>, Box<dyn std::error::Error>> {
            let cx = get_cx!(&instance.cx);

            let value = match value.as_str() {
                Some(v) => v.to_string(),
                None => return Err(QuickSocketError::JsonParseFail.to_box()),
            };

            Ok(JsString::new(cx, value))
        }

        Ok(object(self, value)?)
    }
}

pub fn socket_data_handler(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let interface = cx.argument::<BoxedJsInterface>(0)?;
    let borrowed = match interface.try_borrow_mut() {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };
    borrowed.socket_data_handler(cx)
}
