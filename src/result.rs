use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MtcapError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] json::Error),
    #[error("{0}")]
    Other(String),
}

impl From<MtcapError> for io::Error {
    fn from(err: MtcapError) -> io::Error {
        match err {
            MtcapError::Io(e) => e,
            MtcapError::Json(inner) => io::Error::new(io::ErrorKind::Other, inner),
            MtcapError::Other(inner) => io::Error::new(io::ErrorKind::Other, inner),
        }
    }
}
