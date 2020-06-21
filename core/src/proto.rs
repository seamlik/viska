//! Protocols between [Node](crate::Node)s.

use flexbuffers::DeserializationError;
use http::StatusCode;
use mime::Mime;
use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;

/// Incoming request.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Message(Message),
    Ping,
}

fn is_empty_body(value: &ResponseBody) -> bool {
    if let ResponseBody::None = value {
        true
    } else {
        false
    }
}

fn is_ok_status(code: &StatusCode) -> bool {
    code.as_u16() == 200
}

/// Response to a [Request].
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Response {
    #[serde(with = "serde_with::rust::display_fromstr")]
    #[serde(skip_serializing_if = "is_ok_status")]
    pub status: StatusCode,

    #[serde(skip_serializing_if = "is_empty_body")]
    body: ResponseBody,
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

impl From<DeserializationError> for Response {
    fn from(src: DeserializationError) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: ResponseBody::Text(src.to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum ResponseBody {
    None,
    Text(String),
}

impl Default for ResponseBody {
    fn default() -> Self {
        Self::None
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
