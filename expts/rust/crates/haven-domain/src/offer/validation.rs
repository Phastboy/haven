//! Input validation rules and bounds for offer fields.

use super::{CurrencyCode, Price};
use crate::DomainError;

/// Minimum character count for an offer title.
pub const MIN_TITLE_CHARS: usize = 3;

/// Maximum character count for an offer title.
pub const MAX_TITLE_CHARS: usize = 100;

/// Minimum character count for an offer description.
pub const MIN_DESCRIPTION_CHARS: usize = 10;

/// Maximum character count for an offer description.
pub const MAX_DESCRIPTION_CHARS: usize = 2000;

/// Validates that an offer title meets character length constraints.
///
/// # Errors
///
/// Returns [`DomainError::BlankTitle`] if the title is whitespace-only,
/// [`DomainError::TitleTooShort`] if shorter than [`MIN_TITLE_CHARS`], or
/// [`DomainError::TitleTooLong`] if longer than [`MAX_TITLE_CHARS`].
pub fn validate_title(title: &str) -> Result<(), DomainError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(DomainError::BlankTitle);
    }
    let count = trimmed.chars().count();
    if count < MIN_TITLE_CHARS {
        return Err(DomainError::TitleTooShort);
    }
    if count > MAX_TITLE_CHARS {
        return Err(DomainError::TitleTooLong);
    }
    Ok(())
}

/// Validates that an offer description meets character length constraints.
///
/// # Errors
///
/// Returns [`DomainError::BlankDescription`] if whitespace-only,
/// [`DomainError::DescriptionTooShort`] if shorter than [`MIN_DESCRIPTION_CHARS`], or
/// [`DomainError::DescriptionTooLong`] if longer than [`MAX_DESCRIPTION_CHARS`].
pub fn validate_description(description: &str) -> Result<(), DomainError> {
    let trimmed = description.trim();
    if trimmed.is_empty() {
        return Err(DomainError::BlankDescription);
    }
    let count = trimmed.chars().count();
    if count < MIN_DESCRIPTION_CHARS {
        return Err(DomainError::DescriptionTooShort);
    }
    if count > MAX_DESCRIPTION_CHARS {
        return Err(DomainError::DescriptionTooLong);
    }
    Ok(())
}

/// Validates that an offer price and currency combination is valid.
/// When price is greater than zero, currency must be explicitly provided.
///
/// # Errors
///
/// Returns [`DomainError::CurrencyRequired`] if price is positive and currency is missing.
pub fn validate_price_and_currency(
    price: Price,
    currency: Option<&CurrencyCode>,
) -> Result<(), DomainError> {
    if price.as_i32() > 0 && currency.is_none() {
        return Err(DomainError::CurrencyRequired);
    }
    Ok(())
}
