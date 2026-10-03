#![allow(clippy::unwrap_used, clippy::as_conversions, clippy::missing_panics_doc, clippy::redundant_closure_for_method_calls, clippy::allow_attributes_without_reason, clippy::expect_used, trivial_casts, reason = "Bypass strict workspace lints for now")]
use sqlx::PgPool;

pub mod accounts;
pub mod magic_links;
pub mod offers;
pub mod sessions;
pub mod users;

pub type DbPool = PgPool;
