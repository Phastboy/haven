#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::manual_assert,
    clippy::manual_div_ceil,
    clippy::uninlined_format_args,
    reason = "Test script"
)]

use clap::Parser;
use dotenvy::dotenv;
use haven_db::PostgresRegistry;
use haven_domain::account::Email;
use haven_domain::ports::Registry;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of users to generate
    #[arg(short, long, default_value_t = 10000)]
    users: usize,

    /// Output JSON file for the session tokens
    #[arg(short, long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../haven-web/load-tests/data/sessions.json"))]
    output: PathBuf,
}

#[derive(Serialize)]
struct SessionOutput {
    user_id: String,
    session_token: String,
    offer_ids: Vec<String>,
}

#[tokio::main]
#[allow(clippy::too_many_lines, reason = "orchestration harness")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Args::parse();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let parsed_url = url::Url::parse(&database_url).expect("DATABASE_URL must be a valid URL");
    let host = parsed_url
        .host_str()
        .expect("DATABASE_URL must have a host");
    if host != "localhost" && host != "127.0.0.1" && host != "::1" {
        panic!(
            "Safety check failed: DATABASE_URL host must be exactly localhost, 127.0.0.1, or ::1 (got {host})."
        );
    }

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    let registry = Arc::new(PostgresRegistry::new(&pool));

    println!("Seeding {} users...", args.users);

    let mut sessions = Vec::with_capacity(args.users);

    // We can do this in batches using futures unordered to be faster, but for 10k it's fast enough
    // to just chunk it. We'll use tasks.
    let mut handles = Vec::new();

    let chunk_size = 500;
    for chunk_idx in 0..((args.users + chunk_size - 1) / chunk_size) {
        let registry = registry.clone();
        let start = chunk_idx * chunk_size;
        let end = std::cmp::min(start + chunk_size, args.users);

        let handle = tokio::spawn(async move {
            let mut chunk_sessions = Vec::new();
            for i in start..end {
                let email_str = format!("load-test-user-{}@test.haven.com", uuid::Uuid::new_v4());
                let email = Email::parse(&email_str).expect("valid email");

                let account = registry
                    .accounts()
                    .create(&email)
                    .await
                    .expect("create account");
                registry
                    .accounts()
                    .mark_verified(account.id)
                    .await
                    .expect("mark verified");
                let user = registry
                    .users()
                    .create(account.id)
                    .await
                    .expect("create user");

                let (raw_token, hashed_token) = {
                    let pt = topcoat::session::Token::random();
                    let hash_bytes = pt.hash();

                    let mut hex_string = String::with_capacity(64);
                    for byte in hash_bytes.iter() {
                        use std::fmt::Write;
                        #[allow(
                            clippy::let_underscore_must_use,
                            reason = "writing to in-memory String is infallible"
                        )]
                        let _ = write!(hex_string, "{byte:02x}");
                    }

                    (
                        pt.encode(),
                        haven_domain::session::HashedToken::from_hex(hex_string),
                    )
                };

                let expires_at = chrono::Utc::now() + chrono::Duration::try_days(7).unwrap();
                let _session = registry
                    .sessions()
                    .create(account.id, &hashed_token, expires_at, None, None)
                    .await
                    .expect("create session");

                // Seed offers to make the database realistically populated.
                // Mostly 10 per user, but every 100th user gets 200 offers to test unpaginated tails.
                let mut user_offer_ids = Vec::new();
                let num_offers = if i % 100 == 0 { 200 } else { 10 };

                for j in 0..num_offers {
                    let create_req = haven_domain::offer::CreateOffer {
                        title: format!("Test Offer {j} from User {i}"),
                        description: Some(
                            "This is a seeded offer to fill the database.".to_string(),
                        ),
                        price: haven_domain::offer::Price::new(1000 + j).unwrap(),
                        currency: haven_domain::offer::CurrencyCode::parse("NGN").unwrap(),
                    };
                    let offer = registry
                        .offers()
                        .create(
                            user.id,
                            haven_domain::ports::IdempotencyKey(uuid::Uuid::new_v4()),
                            &create_req,
                        )
                        .await
                        .expect("create offer");

                    user_offer_ids.push(offer.id.to_string());
                }

                chunk_sessions.push(SessionOutput {
                    user_id: user.id.to_string(),
                    session_token: raw_token.as_str().to_string(),
                    offer_ids: user_offer_ids,
                });
            }
            chunk_sessions
        });
        handles.push(handle);
    }

    for handle in handles {
        let chunk_res = handle.await?;
        sessions.extend(chunk_res);
    }

    println!("Vacuuming and analyzing database to ensure accurate statistics...");
    sqlx::query("VACUUM ANALYZE;").execute(&pool).await?;

    println!("Writing sessions to {}", args.output.display());
    if let Some(parent) = args.output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&args.output)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    serde_json::to_writer_pretty(file, &sessions)?;

    println!("Done. Seeded {} sessions.", sessions.len());

    Ok(())
}
