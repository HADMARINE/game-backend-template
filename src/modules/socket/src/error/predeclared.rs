use std::fmt;

use json::{object, JsonValue};

pub struct ErrorDetails {
    code: String,
    message: String,
}

#[derive(Debug)]
pub enum QuickSocketError {
    SocketBufferReadFail,
    VacantPortSearchFail,
    ChannelInitializeFail,
    JsonParseFail,
    JsonFormatInvalid,
    EventNotFound,
    Undefined(String, String),
}

impl QuickSocketError {
    pub fn jsonify(&self) -> JsonValue {
        let details = self.details();
        object! {
            code: details.code,
            message: details.message
        }
    }

    pub fn details(&self) -> ErrorDetails {
        match *self {
            QuickSocketError::SocketBufferReadFail => ErrorDetails {
                code: String::from("SOCKET_BUFFER_READ_FAIL"),
                message: String::from("Failed to read buffer from socket"),
            },
            QuickSocketError::VacantPortSearchFail => ErrorDetails {
                code: String::from("VACANT_PORT_SEARCH_FAIL"),
                message: String::from("Failed to find vacant port"),
            },
            QuickSocketError::ChannelInitializeFail => ErrorDetails {
                code: String::from("CHANNEL_INITIALIZE_FAIL"),
                message: String::from("Failed to initialize channel"),
            },
            QuickSocketError::JsonParseFail => ErrorDetails {
                code: String::from("JSON_PARSE_FAIL"),
                message: String::from("Failed to parse json"),
            },
            QuickSocketError::JsonFormatInvalid => ErrorDetails {
                code: String::from("JSON_FORMAT_INVALID"),
                message: String::from("JSON format is invalid. Did you forgot to send event?"),
            },
            QuickSocketError::EventNotFound => ErrorDetails {
                code: String::from("EVENT_NOT_FOUND"),
                message: String::from("Event not found"),
            },
            QuickSocketError::Undefined(code, message) => ErrorDetails { code, message },
        }
    }
}

impl fmt::Display for QuickSocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let details = self.details();
        write!(f, "{} : {}", details.code, details.message)
    }
}

impl std::error::Error for QuickSocketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
