use thiserror::Error;

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("invalid license format: {0}")]
    Format(String),

    #[error("signature verification failed")]
    BadSignature,

    #[error("license not yet valid")]
    NotYetValid,

    #[error("license expired")]
    Expired,

    #[error("device binding mismatch")]
    DeviceMismatch,
}
