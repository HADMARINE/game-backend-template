use crate::js_interface::JsInterface;

use json::JsonValue;
use std::collections::HashMap;

// pub mod echo;

pub type BridgeMapType = HashMap<String, BridgeHandlerType>;

pub type BridgeHandlerType =
    Box<dyn Fn(JsonValue, &JsInterface) -> Result<(), Box<dyn std::error::Error>>>;

pub fn manager() -> BridgeMapType {
    // return match preset.as_str() {
    //     "none" => HashMap::new(),
    //     "echo" => echo::get(),
    //     _ => {
    //         panic!("Invalid preset : {}", preset);
    //     }
    // };
    let map = HashMap::new(); // TODO : Complete this
    map
}
