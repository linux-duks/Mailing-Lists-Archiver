use std::io::{self};
use std::result;
use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown error")]
    Unknown,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    NNTP(#[from] nntp::NNTPError),

    #[error("Failed reconnecting to NNTP. Exceeded attempts")]
    NNTPReconnectionError,
}
