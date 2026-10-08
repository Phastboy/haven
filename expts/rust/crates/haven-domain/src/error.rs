use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid email: {0}")]
    InvalidEmail(String),

    #[error("invalid price: must be non-negative")]
    InvalidPrice,

    #[error("invalid currency code: must be one of NGN, USD, EUR, GBP, CAD, AUD, KES, GHS")]
    InvalidCurrencyCode,

    #[error("title must not be blank")]
    BlankTitle,

    #[error("title must be at least 3 characters")]
    TitleTooShort,

    #[error("title must not exceed 100 characters")]
    TitleTooLong,

    #[error("description must not be blank")]
    BlankDescription,

    #[error("description must be at least 10 characters")]
    DescriptionTooShort,

    #[error("description must not exceed 2000 characters")]
    DescriptionTooLong,

    #[error("token generation failed")]
    TokenGenerationFailed,
}
