#![allow(unreachable_pub, reason = "binary crate")]

mod app;
mod cx_helpers;

use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use haven_db::PostgresRegistry;
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

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    let registry = Arc::new(PostgresRegistry::new(&pool));
    let state = cx_helpers::AppState { registry };

    let router = app::router()
        .cookies()
        .sessions(SessionConfig::builder()
            .token_store(topcoat::session::cookie::CookieTokenStore::new().name("sid"))
            .build()
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
