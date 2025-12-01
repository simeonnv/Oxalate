use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid protocol")]
    InvalidProtocol(#[from] ParseError),
    #[error("failed to set port in Urls")]
    SetPortError,
}
