use periphos_core::CoreError;
use serde::Serialize;

/// Error shape sent to the frontend: `{ code, message }`.
#[derive(Debug, Serialize)]
pub struct IpcError {
    code: &'static str,
    message: String,
}

impl From<CoreError> for IpcError {
    fn from(err: CoreError) -> Self {
        Self {
            code: err.code(),
            message: err.to_string(),
        }
    }
}

pub type IpcResult<T> = Result<T, IpcError>;
