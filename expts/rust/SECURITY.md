# Haven Security Guardrails

This document outlines the strict security requirements and guardrails that govern the Haven implementation.
Any changes to the codebase must conform to these rules.

## Cross-Site Scripting (XSS)

1. **Auto-Escaping:** We rely on Topcoat's `view!` macro to automatically escape all dynamic content in HTML. 
2. **No Unescaped Content:** The use of `_unescaped` injection methods in Topcoat is strictly forbidden unless accompanied by a written justification and exhaustive manual sanitization.
3. **Content Security Policy (CSP):** The application must send a `Content-Security-Policy: default-src 'self'` header to prevent inline scripts and unauthorized external resources.
4. **Attribute Escaping:** Ensure that user-provided URLs (if ever added) strictly validate their scheme (e.g. `http`/`https` only) to prevent `javascript:` execution inside `href` attributes.
5. **No Reflected Input:** Never blindly echo query parameters (e.g. `?token=`) or untrusted error messages directly into the DOM.
6. **Open Redirects:** Any route accepting a `next` or return URL must restrict redirection to relative paths only.

## Cross-Site Request Forgery (CSRF)

1. **Topcoat OriginPolicy:** We rely on Topcoat's default `OriginPolicy`, which rejects state-changing cross-origin browser requests (e.g. `POST`, `PUT`, `DELETE`) with a `403 Forbidden`.
2. **Cookie Settings:** Session cookies must have `SameSite=Lax` (or `Strict`), `Secure`, `HttpOnly`, and `Path=/`. Topcoat configures this by default.
3. **No State Change on GET:** Never perform state-changing operations on `GET` requests. (e.g. `GET /auth/verify` must render a confirmation page that submits a `POST` request to consume the token).

## Authentication & Enumeration

1. **No Enumeration:** Responses for sign-in or password recovery flows must be identical regardless of whether an account exists. Never reveal if an email is registered.
2. **Resource Probing:** If a resource is requested and the user is not the owner (or the resource does not exist), the response must be `404 Not Found`. Returning a `403 Forbidden` for a missing/unowned resource leaks its existence.
3. **Cookie Security:** The session token must be hashed before storage. Only the client holds the plaintext token.

## Rate Limiting and Abuse Prevention

1. **Sign-In Flow:** `POST /auth/sign-in` must be rate-limited by IP address and by Email (resend cooldown) to prevent mail-bombing.
2. **Write Operations:** Operations like `POST /offers/new` must be rate-limited per user session using a token bucket (e.g., 5 creates burst, 1 per few seconds) to prevent spam.
3. **Verify Flow:** Limit attempts on `/auth/verify` to prevent brute-forcing.
4. **Limits Response:** Rate-limiting responses must not leak account existence.
5. **Magic Link Expiry:** Magic links must expire quickly (e.g., 10-15 minutes) and are single-use. Generating a new link must invalidate any existing unused link.

## Idempotency

1. **Idempotency Keys:** Endpoints that create resources (e.g. `POST /offers/new`) must require an idempotency key. This key should be minted when the form is requested and submitted via a hidden field.
2. **Database Enforcement:** Enforce idempotency at the database level using `UNIQUE (user_id, idempotency_key)` constraints and `ON CONFLICT DO NOTHING`.
3. **No Check-Then-Insert:** Rely on the database constraints to handle concurrency. Do not perform a `SELECT` check before an `INSERT` to ensure uniqueness.
4. **Atomic Consumption:** Consuming single-use tokens must be atomic (e.g., `UPDATE magic_link SET used_at = NOW() WHERE id = $1 AND used_at IS NULL RETURNING *`).
