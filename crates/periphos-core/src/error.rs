use thiserror::Error;

/// Errors shared across the PeriphOS crates.
///
/// Every variant maps to a stable machine-readable [`CoreError::code`] so the
/// frontend can react to specific failures without parsing messages.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{entity} '{id}' was not found")]
    NotFound { entity: &'static str, id: String },

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Conflict(String),

    #[error("this operation is not supported: {0}")]
    Unsupported(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("device error: {0}")]
    Device(String),
}

impl CoreError {
    /// Stable identifier for the error category, exposed over IPC.
    pub fn code(&self) -> &'static str {
        match self {
            CoreError::NotFound { .. } => "not_found",
            CoreError::Validation(_) => "validation",
            CoreError::Conflict(_) => "conflict",
            CoreError::Unsupported(_) => "unsupported",
            CoreError::Storage(_) => "storage",
            CoreError::Device(_) => "device",
        }
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
