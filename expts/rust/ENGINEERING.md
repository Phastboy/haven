
## Documentation rule (mandatory)

Never guess an API. Before using or changing any Topcoat, Rust std, or
dependency API, read the real documentation first:

1. Topcoat guides: find them with
   `cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="topcoat") | .manifest_path'`
   and read the `docs/` folder beside it (router, view, session, cookie,
   context, app_context, functions_not_middlewares, mail). Read the matching
   file before writing code for that area.
2. Exact signatures: grep the crate source in `~/.cargo/registry/src/` or run
   `cargo doc -p topcoat --open`. Source doc comments include working examples;
   copy their idioms.
3. Online fallback (pin the version from Cargo.lock):
   https://docs.rs/topcoat/<version>/topcoat/ and https://github.com/tokio-rs/topcoat
4. Rust and other crates: https://doc.rust-lang.org/std/, https://docs.rs/<crate>/<version>
5. If a compiler error names a missing item, search the docs and source for the
   real name. Do not invent imports, macros, or method names.
6. Security claims (CSRF, cookies, escaping, rate limits) must cite the file
   and line in the docs or source that confirms them.
7. Build incrementally: `cargo check -p haven-web` after each file or route.

## Database & SQLx Compilation

The Postgres database for this rust service runs in a Docker container (`haven-postgres`). If the container is restarted or recreated, its ephemeral data is lost.

Because `sqlx` macros validate SQL against the live database at compile-time, `cargo clippy` and `cargo check` will fail abruptly if the database is missing or empty.

**Mitigation:**
1. We have added `.cargo/config.toml` that forces `SQLX_OFFLINE=true`. This ensures Cargo always builds against the `.sqlx` cache folder rather than requiring a live database.
2. If your database gets wiped or you need to start fresh, run:
   ```bash
   ./bin/setup_db.sh
   ```
   This script will recreate the container, run `cargo sqlx database setup`, and update the `.sqlx` cache.
3. Whenever you add or change SQL queries, you must have a live database running and manually update the cache:
   ```bash
   cargo sqlx prepare --workspace
   ```
