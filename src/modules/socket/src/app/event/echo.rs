use std::collections::HashMap;

use json::JsonValue;

use crate::{
    error::predeclared::QuickSocketError,
    socket_instance::{event::ResponseStatus, ChannelController},
};

use super::EventMapType;

pub fn get() -> EventMapType {
    let mut map: EventMapType = HashMap::new();

    map.insert(String::from("echo"), echo);
    map.insert(String::from("send_to_js"), send_to_js);

    map
}

fn echo(ctrl: ChannelController) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    // ctrl.emit_to(vec![ctrl.accepted_client], ResponseStatus::Ok, ctrl.value);

    Ok(Some(ctrl.value))
}

fn send_to_js(ctrl: ChannelController) -> Result<Option<JsonValue>, Box<QuickSocketError>> {
    // ctrl.emit_to(vec![ctrl.accepted_client], ResponseStatus::Ok, ctrl.value);

    Ok(Some(ctrl.value))
}
