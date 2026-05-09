#[derive(Debug, thiserror::Error)]
pub enum AiLibError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
}

pub type AiLibResult<T> = Result<T, AiLibError>;
