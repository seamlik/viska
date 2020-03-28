//! Protocols between [Node](crate::Node)s.

use http::StatusCode;
use mime::Mime;
use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_cbor::Value;

/// Incoming request.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Message(Message),
    Ping,
}

fn empty_body() -> Value {
    Value::Null
}

fn is_empty_body(value: &Value) -> bool {
    if let Value::Null = value {
        true
    } else {
        false
    }
}

fn is_ok_status(code: &StatusCode) -> bool {
    code.as_u16() == 200
}

/// Response to a [Request].
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Response {
    #[serde(with = "serde_with::rust::display_fromstr")]
    #[serde(skip_serializing_if = "is_ok_status")]
    pub status: StatusCode,

    #[serde(skip_serializing_if = "is_empty_body")]
    body: Value,
}

impl Response {
    /// Creates a response with HTTP status code 403.
    pub fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            ..Default::default()
        }
    }
}

/// Response of "OK".
impl Default for Response {
    fn default() -> Self {
        Self {
            body: empty_body(),
            status: Default::default(),
        }
    }
}

impl From<serde_cbor::Error> for Response {
    fn from(src: serde_cbor::Error) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: Value::Text(src.to_string()),
        }
    }
}

/// Chat message.
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    /// Globally unique ID of this [Message].
    pub id: String,
    pub content: Blob,

    // Recipients of this [Message], excluding the sender itself.
    pub recipients: Vec<ByteBuf>,

    /// Timestamp when this [Message] was sent.
    ///
    /// In seconds with fractional part since [epoch](std::time::UNIX_EPOCH).
    pub time: f64,
}

/// Generic binary data.
#[derive(Debug, Serialize, Deserialize)]
pub struct Blob {
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub mime: Mime,

    pub content: ByteBuf,
}
