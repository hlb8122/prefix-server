use std::fmt;

use actix_web::{error, HttpResponse};
use hex::FromHexError;

use super::jsonrpc_client::ClientError;

#[derive(Debug)]
pub enum ServerError {
    PrefixNotFound,
    InvalidHex,
    Client(ClientError)
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            ServerError::PrefixNotFound => "prefix not found",
            ServerError::InvalidHex => "invalid hex",
            ServerError::Client(_err) => "client error" // TODO: More detail here
        };
        write!(f, "{}", printable)
    }
}

impl From<FromHexError> for ServerError {
    fn from(_err: FromHexError) -> Self {
        // TODO: More detail
        ServerError::InvalidHex
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
            ServerError::InvalidHex => HttpResponse::BadRequest(),
            ServerError::Client(_) => HttpResponse::InternalServerError(),
        }
        .body(self.to_string())
    }
}