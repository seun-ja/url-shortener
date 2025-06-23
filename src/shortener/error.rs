use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Errors produced by the URL shortener service
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum UrlShortenerError {
    /// The given string is not a valid URL
    InvalidUrl,
    /// The provided slug was not found in our records
    NotFound,
    /// The provided custom slug is already being used for a different URL
    SlugAlreadyExists,
    /// Database error
    DatabaseError(String),
    /// Any other service error
    InternalServiceError,
    /// File Open Error
    FileOpenError(String),
    /// Serialization Error
    SerializationError(String),
}

impl Display for UrlShortenerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlShortenerError::InvalidUrl => f.write_str("Invalid URL provided"),
            UrlShortenerError::NotFound => f.write_str("Slug not found"),
            UrlShortenerError::SlugAlreadyExists => {
                f.write_str("Slug is already used for a different URL")
            }
            UrlShortenerError::InternalServiceError => f.write_str("Internal service error"),
            UrlShortenerError::FileOpenError(e) => write!(f, "File Open Error: {}", e),
            UrlShortenerError::SerializationError(e) => write!(f, "File Open Error: {}", e),
            UrlShortenerError::DatabaseError(e) => write!(f, "Database Error: {}", e),
        }
    }
}
