use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("the given token is {0}")]
    InvalidToken(String),
}
