use std::collections::HashMap;

use json::JsonValue;

pub fn manager(
    preset: String,
) -> HashMap<String, Box<dyn FnOnce(JsonValue) -> Result<(), Box<dyn std::error::Error>>>> {
    return match preset.as_str() {
        "none" => HashMap::new(),
        _ => {
            panic!("Invalid preset : {}", preset);
        }
    };
}
