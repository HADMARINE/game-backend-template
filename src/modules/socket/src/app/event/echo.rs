use std::collections::HashMap;

use json::JsonValue;

use crate::js_interface::JsInterface;

use super::EventMapType;

pub fn get() -> EventMapType {
    let mut map = HashMap::new();

    map
}

// fn echo(value: JsonValue, interface: &JsInterface) -> Result<(), Box<dyn std::error::Error>> {}
