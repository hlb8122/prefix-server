use std::fmt;

use hex::FromHexError;

use crate::{bitcoin::streams::StreamError, SETTINGS};

use super::jsonrpc_client::ClientError;

#[derive(Debug)]
pub enum ServerError {
    PrefixNotFound,
    PrefixTooShort,
    InvalidHex(FromHexError),
    Client(ClientError),
    Streaming(StreamError),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            ServerError::PrefixNotFound => "prefix not found",
            ServerError::PrefixTooShort => {
                return write!(f, "prefix too shorter than {} bytes", SETTINGS.min_prefix)
            }
            ServerError::InvalidHex(err) => return err.fmt(f),
            ServerError::Client(_err) => "client error",
            ServerError::Streaming(err) => return err.fmt(f),
        };
        write!(f, "{}", printable)
    }
}

impl From<FromHexError> for ServerError {
    fn from(err: FromHexError) -> Self {
        ServerError::InvalidHex(err)
    }
}

impl From<ClientError> for ServerError {
    fn from(err: ClientError) -> Self {
        ServerError::Client(err)
    }
}
