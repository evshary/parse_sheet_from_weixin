#[derive(Debug, thiserror::Error)]
pub enum SheetError {
    #[error("Failed to get {0}")]
    GetFailed(String),

    #[error("Parse Failed")]
    ParseFailed,
}
