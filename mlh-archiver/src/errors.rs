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

    #[allow(clippy::upper_case_acronyms)]
    #[error(transparent)]
    NNTP(#[from] nntp::NNTPError),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("invalid list selection. At least one should be configured, or selected in runtime")]
    ListSelectionEmpty,
    #[error("configured list(s) not available in server. {} Lists with error: {}", unavailable_lists.len(), unavailable_lists.iter().map(|x| x.to_string() + ",").collect::<String>()
)]
    ConfiguredListsNotAvailable { unavailable_lists: Vec<String> },
    #[error("none of the configured lists are available in server")]
    AllListsUnavailable,

    #[error(transparent)]
    Io(#[from] io::Error),
}
