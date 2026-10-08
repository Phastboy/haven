#![allow(unreachable_pub, reason = "binary crate")]

mod app;
mod cx_helpers;
pub mod token_store;

use haven_db::PostgresRegistry;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::io::Write;
use std::sync::Arc;
use token_store::AdaptiveCookieTokenStore;
use topcoat::{
    cookie::RouterBuilderCookieExt,
    mail::{FileTransport, RouterBuilderMailExt},
    router::RouterBuilderDiscoverExt,
    session::{RouterBuilderSessionExt, SessionConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;

    let pool_size: u32 = env::var("DB_POOL_SIZE")
        .unwrap_or_else(|_| "20".to_string())
        .parse()
        .map_err(|_| "DB_POOL_SIZE must be a number")?;

    if pool_size == 0 {
        return Err("DB_POOL_SIZE must be greater than zero".into());
    }

    writeln!(std::io::stdout(), "Effective DB_POOL_SIZE: {pool_size}").unwrap_or(());

    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    let google_oauth = app::auth::google::GoogleOAuthConfig::from_env()?;
    if google_oauth.is_some() {
        writeln!(std::io::stdout(), "Google OAuth: enabled").unwrap_or(());
    } else {
        writeln!(
            std::io::stdout(),
            "Google OAuth: disabled (credentials not set)"
        )
        .unwrap_or(());
    }

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let registry = Arc::new(PostgresRegistry::new(&pool));
    let state = cx_helpers::AppState {
        registry,
        google_oauth,
        http_client,
    };

    let router = app::router()
        .cookies()
        .sessions(
            SessionConfig::builder()
                .token_store(AdaptiveCookieTokenStore::new().name("sid"))
                .build(),
        )
        .app_context(state)
        .app_context(cx_helpers::SignInLimiter::new())
        .app_context(cx_helpers::CreateOfferLimiter::new())
        .mail(
            topcoat::mail::MailConfig::builder()
                .transport(FileTransport::new("target/mail"))
                .build(),
        )
        .discover()
        .build();

    topcoat::start(router).await?;
    Ok(())
}
