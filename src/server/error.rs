extern crate thiserror;

use core::fmt;

#[derive(Error, Debug)]
pub struct AError {
    message: String
}

impl fmt::Display for AError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "incomplete utf-8 byte sequence from index {}", self.message)
    }
}

impl AError {
    pub fn new(message: &str) -> AError {
        AError { message: message.to_owned() }
    }

    pub fn default() -> AError {
        AError { message: String::from("系统异常") }
    }

    pub fn service(message: &str) -> AError {
        AError { message: message.to_owned() }
    }

    pub fn parameter(message: &str) -> AError {
        AError { message: message.to_owned() }
    }

    pub fn p() -> AError {
        AError { message: "参数异常".to_owned() }
    }

    pub fn info(self) -> String {
        self.message.to_owned()
    }
}


#[derive(Error, Debug)]
pub enum XError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),



    #[error("data store disconnected 2")]
    AError(#[from] AError),
    #[error("data store disconnected 3")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("data store disconnected 4")]
    JsonError(#[from] serde_json::Error),
    #[error("data store disconnected 5")]
    MultipartError(#[from] actix_multipart::MultipartError),

    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}