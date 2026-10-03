mod app;
mod cx_helpers;

use sqlx::postgres::PgPoolOptions;
use std::env;
use topcoat::{
    cookie::RouterBuilderCookieExt,
    mail::{FileTransport, RouterBuilderMailExt},
    router::{RouterBuilderDiscoverExt, module_router},
    session::{RouterBuilderSessionExt, SessionConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    let router = module_router!()
        .cookies()
        .sessions(SessionConfig::default())
        .app_context(pool)
        .app_context(cx_helpers::SignInLimiter::new())
        .app_context(cx_helpers::CreateOfferLimiter::new())
        .mail(
            topcoat::mail::MailConfig::builder()
                .transport(FileTransport::new("target/mail"))
                .build(),
        )
        .discover()
        .build();

    topcoat::start(router).await.unwrap();
    Ok(())
}
