use crate::socket_instance::ChannelController;

use json::JsonValue;
use std::collections::HashMap;

pub mod echo;

pub type EventMapType = HashMap<String, EventHandlerType>;

pub type EventHandlerType =
    Box<dyn FnOnce(JsonValue, &ChannelController) -> Result<(), Box<dyn std::error::Error>>>;

pub fn manager(preset: String) -> EventMapType {
    return match preset.as_str() {
        "none" => HashMap::new(),
        "echo" => echo::get(),
        _ => {
            panic!("Invalid preset : {}", preset);
        }
    };
}
