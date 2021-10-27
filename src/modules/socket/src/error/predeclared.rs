use std::fmt;

use json::{object, JsonValue};

use crate::socket_instance::ChannelClient;

#[derive(Debug, Clone)]
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
    InternalServerError,
    InstanceInitializeInvalid,
    ClientDataInvalid,
    DataResponseFail,
    EventAlreadyExists,
    ClientAlreadyExists,
    ClientNotRegistered,
    ConnectionClosed(ChannelClient),
    Undefined(String),
    Custom(String, String),
}

impl QuickSocketError {
    pub fn jsonify(&self) -> JsonValue {
        let details = self.details();
        object! {
            code: details.code,
            message: details.message
        }
    }

    pub fn to_box(&'static self) -> Box<dyn std::error::Error> {
        Box::new(self.clone())
    }

    pub fn details(&self) -> ErrorDetails {
        match &*self {
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
            QuickSocketError::InternalServerError => ErrorDetails {
                code: String::from("INTERNAL_SERVER_ERROR"),
                message: String::from("Internal server error"),
            },
            QuickSocketError::InstanceInitializeInvalid => ErrorDetails {
                code: String::from("INSTANCE_INITIALIZE_INVALID"),
                message: String::from("Instance initialize invalid."),
            },
            QuickSocketError::Undefined(message) => ErrorDetails {
                code: String::from("UNDEFINED"),
                message: message.clone(),
            },
            QuickSocketError::Custom(code, message) => ErrorDetails {
                code: code.clone(),
                message: message.clone(),
            },
            QuickSocketError::ClientDataInvalid => ErrorDetails {
                code: String::from("CLIENT_DATA_INVALID"),
                message: String::from("Client data is invalid"),
            },
            QuickSocketError::DataResponseFail => ErrorDetails {
                code: String::from("DATA_RESPONSE_FAIL"),
                message: String::from("Failed to response data"),
            },
            &QuickSocketError::EventAlreadyExists => ErrorDetails {
                code: String::from("EVENT_ALREADY_EXISTS"),
                message: String::from("Event already exists"),
            },
            QuickSocketError::ClientAlreadyExists => ErrorDetails {
                code: String::from("CLIENT_ALREADY_EXISTS"),
                message: String::from("Client already exists"),
            },
            QuickSocketError::ClientNotRegistered => ErrorDetails {
                code: String::from("CLIENT_NOT_REGISTERED"),
                message: String::from("Client is not registered"),
            },
            QuickSocketError::ConnectionClosed(v) => ErrorDetails {
                code: String::from("CONNECTION_CLOSED"),
                message: String::from(format!("Connection closed (UID : {})", &v.uid)),
            },
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
