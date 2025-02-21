use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConversionError {
    FileError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidFormat(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::FileError(e) => write!(f, "File error: {}", e),
            ConversionError::ParseError(e) => write!(f, "Parse error: {}", e),
            ConversionError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl Error for ConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConversionError::FileError(e) => Some(e),
            ConversionError::ParseError(e) => Some(e),
            ConversionError::InvalidFormat(_) => None,
        }
    }
}