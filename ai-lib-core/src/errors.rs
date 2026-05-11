#[derive(Debug, thiserror::Error)]
pub enum AiLibError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("HTTP {status}: {body}")]
    HttpStatus { status: reqwest::StatusCode, body: String },
}

pub type AiLibResult<T> = Result<T, AiLibError>;
