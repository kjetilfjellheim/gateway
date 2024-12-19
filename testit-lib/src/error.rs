/**
 * This module contains the error type for the application.
 */
#[derive(Debug)]
pub enum ApplicationError {
      FileError(String),
      MissingId(String),
      CouldNotFindTest(String),
      ConfigurationError(String),
      ServerStartUpError(String),
}

/**
 * Convert the error to a string.
 */
impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApplicationError::FileError(err) => write!(f, "File error: {}", err),
            ApplicationError::MissingId(err) => write!(f, "Missing id: {}", err),
            ApplicationError::CouldNotFindTest(err) => write!(f, "Could not find test: {}", err),
            ApplicationError::ConfigurationError(err) => write!(f, "Configuration error: {}", err),
            ApplicationError::ServerStartUpError(err) => write!(f, "Server start up error: {}", err),
        }
    }
}