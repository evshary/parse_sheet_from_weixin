#[derive(Debug, thiserror::Error)]
pub enum SheetError {
    #[error("Parse Failed")]
    ParseFailed,
}
