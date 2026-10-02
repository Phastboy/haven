use sqlx::PgPool;

pub mod accounts;
pub mod magic_links;
pub mod offers;
pub mod sessions;
pub mod users;

pub type DbPool = PgPool;
