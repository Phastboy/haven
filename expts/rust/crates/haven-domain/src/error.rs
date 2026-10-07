use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid email: {0}")]
    InvalidEmail(String),

    #[error("invalid price: must be non-negative")]
    InvalidPrice,

    #[error("invalid currency code: must be exactly 3 uppercase ASCII letters")]
    InvalidCurrencyCode,

    #[error("title must not be blank")]
    BlankTitle,

    #[error("title must not exceed 255 characters")]
    TitleTooLong,

    #[error("token generation failed")]
    TokenGenerationFailed,
}
