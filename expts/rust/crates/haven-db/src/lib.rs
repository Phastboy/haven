pub mod accounts;
pub mod magic_links;
pub mod offers;
pub mod sessions;
pub mod users;

use haven_domain::ports::{
    AccountRepository, MagicLinkRepository, OfferRepository, Registry, SessionRepository,
    UserRepository,
};
use sqlx::PgPool;

pub type DbPool = PgPool;

pub struct PostgresRegistry {
    offers: Box<dyn OfferRepository>,
    accounts: Box<dyn AccountRepository>,
    users: Box<dyn UserRepository>,
    magic_links: Box<dyn MagicLinkRepository>,
    sessions: Box<dyn SessionRepository>,
}

impl PostgresRegistry {
    pub fn new(pool: &DbPool) -> Self {
        Self {
            offers: Box::new(offers::PostgresOfferRepository { pool: pool.clone() }),
            accounts: Box::new(accounts::PostgresAccountRepository { pool: pool.clone() }),
            users: Box::new(users::PostgresUserRepository { pool: pool.clone() }),
            magic_links: Box::new(magic_links::PostgresMagicLinkRepository { pool: pool.clone() }),
            sessions: Box::new(sessions::PostgresSessionRepository { pool: pool.clone() }),
        }
    }
}

impl Registry for PostgresRegistry {
    fn offers(&self) -> &(dyn OfferRepository + 'static) {
        self.offers.as_ref()
    }
    fn accounts(&self) -> &(dyn AccountRepository + 'static) {
        self.accounts.as_ref()
    }
    fn users(&self) -> &(dyn UserRepository + 'static) {
        self.users.as_ref()
    }
    fn magic_links(&self) -> &(dyn MagicLinkRepository + 'static) {
        self.magic_links.as_ref()
    }
    fn sessions(&self) -> &(dyn SessionRepository + 'static) {
        self.sessions.as_ref()
    }
}

/// Map a `sqlx::Error` to a `RepoError`
pub(crate) fn map_sqlx_err(e: sqlx::Error) -> haven_domain::ports::RepoError {
    use haven_domain::ports::RepoError;
    match e {
        sqlx::Error::RowNotFound => RepoError::NotFound,
        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                RepoError::Conflict
            } else {
                RepoError::Unavailable(db_err.message().to_string())
            }
        }
        _ => RepoError::Unavailable(e.to_string()),
    }
}
