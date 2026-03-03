use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnterpriseError {
    #[error("enterprise feature unavailable: {0}")]
    FeatureUnavailable(&'static str),

    #[error("enterprise module not linked/loaded")]
    NoEnterpriseModule,

    #[error("license missing or invalid")]
    LicenseInvalid,

    #[error("enterprise operation failed: {0}")]
    OperationFailed(String),
}
