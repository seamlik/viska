use crate::endpoint::ConnectionInfo;
use crate::pki::CertificateId;
use crate::proto::Request;
use crate::proto::Response;
use crate::Connection;
use http::StatusCode;
use quinn::ReadToEndError;
use quinn::RecvStream;
use quinn::SendStream;
use quinn::WriteError;
use std::net::SocketAddr;
use std::sync::Arc;

pub const MAX_PACKET_SIZE_BYTES: usize = 1024 * 1024;

pub struct ResponseWindow {
    connection: Arc<Connection>,
    pub request: Request,
    sender: SendStream,
}

impl ResponseWindow {
    pub async fn new(
        connection: Arc<Connection>,
        mut sender: SendStream,
        receiver: RecvStream,
    ) -> Option<Self> {
        match receiver.read_to_end(MAX_PACKET_SIZE_BYTES).await {
            Ok(raw) => match flexbuffers::from_slice(&raw) {
                Ok(request) => {
                    log::debug!("Received request: {:?}", &request);
                    Some(Self {
                        connection,
                        request,
                        sender,
                    })
                }
                Err(err) => {
                    log::error!("Failed to parse an incoming request: {:?}", &err);
                    send_response(&mut sender, &err.into())
                        .await
                        .unwrap_or_else(|err| {
                            log::error!(
                                "Failed to send a response regarding bad request: {:?}",
                                err
                            )
                        });
                    None
                }
            },
            Err(err) => match err {
                ReadToEndError::TooLong => {
                    sender.reset(StatusCode::PAYLOAD_TOO_LARGE.as_u16().into());
                    connection.close(StatusCode::PAYLOAD_TOO_LARGE);
                    None
                }
                ReadToEndError::Read(inner) => {
                    log::error!("Failed to read an incoming request: {:?}", inner);
                    None
                }
            },
        }
    }

    pub async fn send_response(mut self, response: Response) -> Result<(), WriteError> {
        send_response(&mut self.sender, &response).await
    }

    pub fn disconnect(mut self, err: crate::handler::Error) {
        let code = match err {
            crate::handler::Error::PeerIdAbsent => StatusCode::UNAUTHORIZED,
        };
        self.sender.reset(code.as_u16().into());
        self.connection.close(code);
    }
}

impl ConnectionInfo for ResponseWindow {
    fn account_id(&self) -> Option<CertificateId> {
        self.connection.account_id()
    }
    fn remote_address(&self) -> SocketAddr {
        self.connection.remote_address()
    }
}

async fn send_response(sender: &mut SendStream, response: &Response) -> Result<(), WriteError> {
    log::debug!("Sending response: {:?}", &response);
    let raw = flexbuffers::to_vec(response).expect("Failed to encode a response");
    send_raw(sender, &raw).await
}

async fn send_raw(sender: &mut SendStream, response: &[u8]) -> Result<(), WriteError> {
    sender.write_all(&response).await?;
    sender.finish().await
}
