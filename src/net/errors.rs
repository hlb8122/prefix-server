use std::fmt;

use actix_web::{error, HttpResponse};
use hex::FromHexError;

use super::jsonrpc_client::ClientError;

#[derive(Debug)]
pub enum ServerError {
    PrefixNotFound,
    InvalidHex(FromHexError),
    Client(ClientError),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            ServerError::PrefixNotFound => "prefix not found",
            ServerError::InvalidHex(err) => return err.fmt(f),
            ServerError::Client(_err) => "client error", // TODO: More detail here
        };
        write!(f, "{}", printable)
    }
}

impl From<FromHexError> for ServerError {
    fn from(err: FromHexError) -> Self {
        // TODO: More detail
        ServerError::InvalidHex(err)
    }
}

impl From<ClientError> for ServerError {
    fn from(err: ClientError) -> Self {
        ServerError::Client(err)
    }
}

impl error::ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServerError::PrefixNotFound => HttpResponse::BadRequest(),
            ServerError::InvalidHex(_) => HttpResponse::BadRequest(),
            ServerError::Client(_) => HttpResponse::InternalServerError(),
        }
        .body(self.to_string())
    }
}
