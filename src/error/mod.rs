use std::fmt;

#[derive(Debug)]
pub enum ScoreError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl std::error::Error for ScoreError {}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoreError::IoError(e) => write!(f, "IO error: {}", e),
            ScoreError::SerdeError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl From<std::io::Error> for ScoreError {
    fn from(err: std::io::Error) -> Self {
        ScoreError::IoError(err)
    }
}

impl From<serde_json::Error> for ScoreError {
    fn from(err: serde_json::Error) -> Self {
        ScoreError::SerdeError(err)
    }
}