# haven-db Guardrails

This document outlines the strict guidelines governing the `haven-db` crate. The database is a core platform resource, and its queries must be predictable, observable, and bounded.

## 1. Connection Pooling & Observability

- **Pool Configuration**: `DbPool` (alias for `sqlx::PgPool`) must be properly configured with min/max connections and idle timeouts suited to the environment. It is injected into Axum state.
- **Observability**: 
  - Slow-query measurement and logging must be established.
  - Load testing must validate DB behavior and throughput, not just API responsiveness.
  - Connection pool saturation should be monitored as a key metric.

## 2. Query Shape & Access Paths

- **No Unbounded `SELECT`**: Every query returning a collection must have an explicit pagination or limit strategy.
- **Bounded `fetch_all`**: `fetch_all()` is permitted only when the result set is demonstrably bounded by domain rules.
- **Targeted Columns**: Prefer selecting explicit required columns rather than `SELECT *` where the row might become large.
- **Access Paths**: Every frequently executed query must have an intentional access path. Potentially expensive queries must be verified against query plans (`EXPLAIN` / `EXPLAIN ANALYZE`).

## 3. N+1 Query Prevention

**Rule**: No database query may be executed once per item of an in-memory collection when the operation can be expressed as a bounded batch query or a relational join.

*Bad (N+1 Query Pattern)*:
```rust
for user_id in user_ids {
    find_user(user_id).await?;
}
```

*Good (Batch / Relational)*:
```sql
-- Batch lookup
SELECT * FROM users WHERE id = ANY($1)

-- Join
SELECT * FROM offers o JOIN users u ON u.id = o.user_id WHERE ...
```
CPU-only transformations in loops are perfectly legitimate; it is the repeated *query pattern* that is prohibited.

## 4. Strategic Indexing

Require indexes on all Foreign Keys. This is critical for:
- Parent deletion checks enforcing constraints on child rows.
- Efficient `JOIN` operations.
- Filtering by the foreign key (e.g. finding all offers for a specific user).
- Relationship lookups.

Indexes should use the naming convention `idx_table_column`.

## 5. Transactions

Transactions must remain short. 

**Rule**: Do not perform external I/O (such as API calls, sending emails, or file uploads) while a transaction is open. 

*Good*:
```text
BEGIN
  DB work
COMMIT
external API / email
```

## 6. Schema Evolution

- Every schema change must go through a migration.
- Never manually modify the production schema.
- Migrations must be sequential, forward-only, and descriptive.
- Indexes must be justified by query patterns.
- Avoid destructive migrations without an explicit migration strategy.
- Do not combine unrelated schema changes in one migration.

**Critical**: Application code must not silently depend on schema that hasn't been represented by a migration. This ensures `haven-db` remains reproducible.

## 7. DB / API Boundary

The `haven-db` crate owns:
- SQL queries
- Transactions
- Mapping DB rows to domain representations
- Database-specific error mapping
- Pool management

The `haven-db` crate **does not own**:
- HTTP, routing, or Axum
- Authentication policy
- External APIs or email delivery
- Business workflows (e.g., `magic_links::create` creates a DB record; it does not send the email).

## 8. File Structure (The 80-Line Signal)

**Rule**: DB implementation files should normally remain below 80 lines.

- **80 lines is a strong default, not an absolute ceiling.** It is a signal for architectural inspection of cohesion. 
- Files should remain small to preserve local reasoning and human readability. 
- Exceeding 80 lines is permitted only when its contents form a genuinely cohesive unit and splitting it would introduce artificial fragmentation.
- **Splitting a file solely to reduce line count is not considered compliance.** Extraction into sub-modules (e.g., `offers/create.rs`, `offers/find.rs`) must be based on natural boundaries of responsibility.
