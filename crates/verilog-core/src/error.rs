use thiserror::Error;

#[derive(Debug, Error)]
pub enum VeriLogError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("postcard error: {0}")]
    Postcard(String),

    #[error("invalid store format: {0}")]
    Format(String),

    #[error("signature error: {0}")]
    Signature(String),

    #[error("integrity verification failed: {0}")]
    Integrity(String),
}

impl From<postcard::Error> for VeriLogError {
    fn from(e: postcard::Error) -> Self {
        VeriLogError::Postcard(e.to_string())
    }
}
