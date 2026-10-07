# Data access: SQL injection, N+1, IDOR, query performance

Scope: `haven-db` and any code that builds or runs SQL or loads records for a request.

## SQL injection

**Rule:** user-controlled values reach SQL only as bind parameters.

Check:
- [ ] No `format!`, string concatenation, or `push_str` building SQL with values from request data, config, or the database itself.
- [ ] Queries use `sqlx::query!` / `query_as!` or `query(...).bind(...)`. For dynamic queries, `QueryBuilder` with `push_bind` for values.
- [ ] **Identifiers cannot be bound** (column names, `ORDER BY` field, sort direction, table names). If user input selects one, map it through a fixed allowlist (an enum), never interpolate the raw string.
- [ ] `LIKE` patterns: bind the value, and escape `%`/`_` if a literal match is intended.
- [ ] `IN` lists use `= ANY($1)` with a bound array, not generated placeholder strings from user data.
- [ ] No raw SQL execution of migration-like strings from user input.

Search aid: `rg -n 'format!\(.*(SELECT|INSERT|UPDATE|DELETE|WHERE)' crates/` and `rg -n 'push_str' crates/haven-db`.

## IDOR (insecure direct object reference / broken object-level authorization)

**Rule:** a valid id is never proof of permission. Ownership or access is enforced for every read, update, and delete of a user-owned record.

Check:
- [ ] Every query that fetches or mutates a user-owned record includes the owner or tenant condition **in SQL**: `WHERE id = $1 AND owner_id = $2`. Do not fetch by id and check ownership afterwards in separate code that might be skipped.
- [ ] The acting user id comes from the authenticated session, **never** from the request body, path, or a hidden form field.
- [ ] List endpoints are scoped to the caller; search and export endpoints too.
- [ ] Nested resources check the whole chain (a comment id under another user's post id).
- [ ] Unauthorized access to someone else's record and a missing record are indistinguishable to the client (same status and body, usually 404), to avoid leaking which ids exist.
- [ ] Ids are not relied on as secrets. Random UUIDs reduce guessing but are not authorization.
- [ ] Bulk and batch operations check every id in the batch.
- [ ] Mass assignment: update handlers accept only the fields the user may change (no `role`, `owner_id`, `is_admin` coming from request data).

Test: for each user-owned route, a test where user B requests user A's record and must not see or change it. This matches the k6 authorization script's purpose.

## N+1 queries

**Rule:** the number of queries per request must not grow with the number of rows.

Signs:
- A query inside a `for` / `.map()` / `iter()` loop, often `.await` in the loop body.
- A list fetch followed by a per-item fetch for related data (author, counts, tags).
- Repository methods that look cheap but run one query per call and get called per item.

Fixes, in order of preference:
1. A `JOIN` that returns related data in one query.
2. A batch fetch: `WHERE id = ANY($1)` for all ids, then group in memory.
3. Aggregate in SQL (`COUNT`, `GROUP BY`) instead of loading rows to count them.
4. Denormalized or cached counts if the contract demands it (document the consistency trade-off).

Check:
- [ ] List and feed handlers: count the queries a request makes for N=1, N=10, N=100. The count must be constant.
- [ ] A test that asserts query count for list endpoints (log queries, or wrap the pool in test builds).

## Other query and performance problems

Tie findings to `PERFORMANCE_CONTRACT.md` (RPS, p95/p99 targets).

- [ ] **Unbounded results:** every list query has a `LIMIT` and a stable `ORDER BY`; pagination exists. Prefer keyset (cursor) pagination over large `OFFSET`.
- [ ] **Missing indexes** for columns used in `WHERE`, `JOIN`, `ORDER BY`. Verify with `EXPLAIN (ANALYZE, BUFFERS)` on realistic data; do not guess.
- [ ] **`SELECT *`** on wide rows when few columns are needed.
- [ ] **Connection handling:** do not hold a pooled connection across slow non-database work (HTTP calls, hashing, sleeping). Keep transactions short. Pool size must fit the database plan's connection limit.
- [ ] **Blocking in async:** CPU-heavy or blocking work (password hashing, large serialization, sync file IO) must not run on the async executor threads; use `spawn_blocking`.
- [ ] **Hot path allocations:** needless `clone()`, `format!` in loops, building large `String`s per request, serializing then re-parsing.
- [ ] **Response size:** compress where appropriate and avoid returning data the client does not use.
- [ ] **Transactions:** multi-step writes that must be atomic are in a transaction, with correct error rollback.

Never claim a performance improvement without a measurement (a before/after benchmark or `EXPLAIN` output).
