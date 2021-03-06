//! Communication protocol between [Node](crate::Node)s.

tonic::include_proto!("viska.proto");

use http::StatusCode;
use prost::DecodeError;

impl Response {
    /// Creates a response with HTTP status code 403.
    pub fn forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN.as_u16().into(),
            ..Default::default()
        }
    }

    pub fn bad_request(reason: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST.as_u16().into(),
            reason,
        }
    }

    pub fn from_decode_error(src: &DecodeError) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST.as_u16().into(),
            reason: format!("{}", src),
        }
    }
}

impl From<crate::handler::Error> for Response {
    fn from(src: crate::handler::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16().into(),
            reason: format!("{}", src),
        }
    }
}
