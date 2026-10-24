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

/// Profile validation and format errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ProfileError {
    #[error("display name must not be blank")]
    BlankDisplayName,

    #[error("display name must be at least 2 characters")]
    DisplayNameTooShort,

    #[error("display name must not exceed 50 characters")]
    DisplayNameTooLong,

    #[error("handle must be at least 3 characters")]
    HandleTooShort,

    #[error("handle must not exceed 30 characters")]
    HandleTooLong,

    #[error("handle may only contain lowercase letters, numbers, and hyphens")]
    InvalidHandleFormat,

    #[error("handle '{0}' is reserved")]
    ReservedHandle(String),

    #[error("bio must not exceed 500 characters")]
    BioTooLong,
}

/// Image validation and format errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ImageValidationError {
    #[error("unsupported image format: must be JPEG, PNG, or WebP")]
    UnsupportedFormat,

    #[error("image file exceeds 5 MB limit")]
    FileTooLarge,

    #[error("image file is empty")]
    EmptyFile,

    #[error("invalid image position {0}: must be between 1 and 5")]
    InvalidPosition(u8),

    #[error("image dimensions must be positive integers")]
    InvalidDimensions,

    #[error("invalid image public ID: {0}")]
    InvalidPublicId(String),

    #[error("an offer cannot exceed 5 images")]
    MaxImagesExceeded,

    #[error("an offer must have at least 1 image to be public")]
    MinImagesRequired,
}

/// Filter validation and format errors.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum FilterValidationError {
    #[error("search query must not be blank")]
    BlankSearchQuery,

    #[error("search query must not exceed 100 characters")]
    SearchQueryTooLong,

    #[error("at least one of minimum price or maximum price must be specified")]
    EmptyPriceFilter,

    #[error("minimum price cannot be greater than maximum price")]
    InvalidPriceRange,

    #[error("currency is required when filtering by price")]
    CurrencyRequired,
}

/// Unified domain error type composed from concern-specific error enums.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum DomainError {
    #[error(transparent)]
    Account(#[from] AccountError),

    #[error(transparent)]
    Offer(#[from] OfferValidationError),

    #[error(transparent)]
    Filter(#[from] FilterValidationError),

    #[error(transparent)]
    Session(#[from] SessionError),

    #[error(transparent)]
    Cursor(#[from] CursorError),

    #[error(transparent)]
    Profile(#[from] ProfileError),

    #[error(transparent)]
    Image(#[from] ImageValidationError),

    #[error(transparent)]
    Conversation(#[from] crate::conversation::ConversationDomainError),
}

pub use crate::conversation::{ConversationDomainError, MessageValidationError, SelfConversation};
