# Haven Rust Security Checklist & Audit Heuristics

This reference contains specific audit patterns and grep recipes for reviewing code in `haven-domain`, `haven-db`, and `haven-web`.

---

## 1. XSS & Template Escaping Heuristics

### Audit Checks
- Check that all dynamic content in `topcoat::view::view!` uses standard variable interpolation `(variable)`, which auto-escapes.
- Search for any raw HTML or bypass methods:
  ```bash
  rg "_unescaped" crates/haven-web/
  ```
- Search for open redirects in `GET /auth/verify` or redirect responses:
  ```bash
  rg "redirect\(" crates/haven-web/
  ```
  Ensure all redirect targets start with `/` and do not start with `//` or external schemes (`http://`, `https://`).

---

## 2. CSRF & Request Method Safety

### Audit Checks
- Ensure every `#[topcoat::router::page]` or `#[topcoat::router::endpoint]` responding to `GET` is read-only.
- If an email verification or magic link flow uses a link, the `GET` route must only display a confirmation form with a `<form method="POST">` button.
- Verify `OriginPolicy` is active in `topcoat::router::RouterBuilder`.

---

## 3. Tenancy, Ownership & Anti-Enumeration

### The "404 vs 403" Rule
- If user `A` requests an offer owned by user `B`:
  - **CORRECT**: Return `404 Not Found` (or `DomainError::NotFound`).
  - **VULNERABLE**: Returning `403 Forbidden` confirms that the offer ID exists.
- In `crates/haven-db/src/offers.rs`:
  ```sql
  SELECT * FROM offer WHERE id = $1 AND user_id = $2
  ```
  Query must always include `AND user_id = $2` on owner-scoped operations, rather than selecting by `id` first and checking ownership in Rust.

### Sign-In Enumeration
- `POST /auth/sign-in` must return the exact same success response (e.g. redirect to `/auth/sign-in/sent`) whether the email was registered or not.

---

## 4. Idempotency & Database Atomicity

### No Check-Then-Insert (TOCTOU)
- **Vulnerable Pattern**:
  ```rust
  // WRONG: Race condition between SELECT and INSERT
  if repo.find_by_key(&key).await?.is_some() {
      return Ok(existing);
  }
  repo.insert(&key).await?;
  ```
- **Safe Pattern**:
  ```sql
  INSERT INTO offer (id, user_id, idempotency_key, title, ...)
  VALUES ($1, $2, $3, $4, ...)
  ON CONFLICT (user_id, idempotency_key) DO NOTHING
  RETURNING *;
  ```

### Atomic Token Consumption
- Magic link consumption must be atomic:
  ```sql
  UPDATE magic_link
  SET used_at = NOW()
  WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
  RETURNING *;
  ```

---

## 5. Token & Session Cryptography

- Verify that plaintext session tokens are generated using cryptographically secure RNG (`rand::RngCore` or `getrandom`).
- Verify tokens are hashed with SHA-256 (`sha2::Sha256`) before being written to PostgreSQL.
- Search for accidental leakage of `PlaintextToken`:
  - `PlaintextToken` must implement `Display` by outputting `[REDACTED]`.
  - Only `HashedToken` should be serializable for database storage.

---

## 6. Denial of Service & Rate Limiting

- Check that memory buffers have bounded size limits.
- Check that `SignInLimiter` and `CreateOfferLimiter` are applied in `app_context` on relevant routes.
- Check that string inputs (offer titles, descriptions) validate upper length limits before database writes.
