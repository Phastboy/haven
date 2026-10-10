use thiserror::Error;

/// Account-specific domain errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AccountError {
    #[error("invalid email: {0}")]
    InvalidEmail(String),
}

/// Offer-specific validation and format errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum OfferValidationError {
    #[error("price is required")]
    PriceRequired,

    #[error("invalid price: must be non-negative")]
    InvalidPrice,

    #[error("currency is required when price is greater than zero")]
    CurrencyRequired,

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

    #[error("invalid offer slug: {0}")]
    InvalidSlug(String),
}

/// Session credential errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum SessionError {
    #[error("token generation failed")]
    TokenGenerationFailed,
}

/// Pagination cursor parsing errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum CursorError {
    #[error("invalid pagination cursor")]
    InvalidCursor,
}

/// Unified domain error type composed from concern-specific error enums.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum DomainError {
    #[error(transparent)]
    Account(#[from] AccountError),

    #[error(transparent)]
    Offer(#[from] OfferValidationError),

    #[error(transparent)]
    Session(#[from] SessionError),

    #[error(transparent)]
    Cursor(#[from] CursorError),
}
