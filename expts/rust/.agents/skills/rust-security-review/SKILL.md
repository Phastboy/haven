---
name: rust-security-review
description: >-
  Audits Rust and Topcoat web code for security vulnerabilities, invariant
  violations, authentication/session leaks, XSS, CSRF, account enumeration,
  missing idempotency, or rate-limiting bypasses. Use when reviewing code
  changes, writing security-sensitive handlers, auditing SQL queries or session
  management, or verifying compliance with SECURITY.md.
---

# Rust Security Review Skill (`rust-security-review`)

This skill governs security audits and code reviews for the Haven Rust workspace, enforcing the mandatory guardrails established in [`SECURITY.md`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/SECURITY.md).

---

## 1. The Six Security Pillars

Whenever authoring, editing, or reviewing code in `haven-domain`, `haven-db`, or `haven-web`, audit against these 6 pillars:

### A. Cross-Site Scripting (XSS)
- **Topcoat Auto-Escaping**: All dynamic template values inside `view!` must be auto-escaped.
- **Forbidden Unescaped Methods**: Any call to `_unescaped` is strictly forbidden unless accompanied by a written justification and exhaustive manual sanitization.
- **CSP Headers**: Verify that the CSP layer (`Content-Security-Policy: default-src 'self'`) is attached to router pipelines.
- **Safe Attribute URLs**: Dynamic `href` attributes must be relative or strictly validate `http:`/`https:` schemes (never allow `javascript:` execution).
- **No Reflected Input**: Never reflect query parameters (e.g. `?token=`) or user errors directly into the DOM.
- **Open Redirects**: Any `next` or return URL parameter must be strictly validated as a relative path starting with `/` (reject `//`, `https://`, etc.).

### B. Cross-Site Request Forgery (CSRF)
- **OriginPolicy**: State-changing requests (`POST`, `PUT`, `DELETE`) must enforce Topcoat's `OriginPolicy`.
- **Cookie Flags**: Session cookies must have `SameSite=Lax` (or `Strict`), `Secure`, `HttpOnly`, and `Path=/`.
- **No State Change on GET**: Never perform mutations or token consumption on `GET`. (e.g. `GET /auth/verify` only renders a confirmation page; the actual consumption must be a `POST`).

### C. Authentication & Anti-Enumeration
- **No User Enumeration**: Responses for sign-in or magic link generation must be completely identical whether an email exists or not. Never reveal account existence.
- **Anti-Probing 404s**: When accessing a resource that does not belong to the authenticated user, return `404 Not Found` (never `403 Forbidden`). Returning 403 leaks that the resource ID exists.
- **Hashed Session Tokens**: Raw session tokens (`PlaintextToken`) live exclusively in client cookies. Only SHA-256 hashes (`HashedToken`) are stored in PostgreSQL.

### D. Rate Limiting & Abuse Prevention
- **Sign-In Flow**: `POST /auth/sign-in` must be rate-limited by IP address and email cooldown to prevent mail-bombing.
- **Write Operations**: `POST /offers/new` and other mutations must be bounded per session via a token bucket (e.g. burst limit with refill).
- **Rate Limit Responses**: Rate-limit errors must not leak account existence.
- **Magic Link Lifespan**: Magic links must expire quickly (10-15 minutes) and are strictly single-use. Minting a new magic link must invalidate previous unused links.

### E. Idempotency & Database Concurrency
- **Idempotency Keys**: Endpoints creating resources (e.g. `POST /offers/new`) require an idempotency key submitted from the form.
- **Database-Level Enforcement**: Idempotency must be guaranteed via PostgreSQL `UNIQUE (user_id, idempotency_key)` and `ON CONFLICT DO NOTHING`.
- **No Check-Then-Insert**: Never use `SELECT` followed by `INSERT` in application code. Rely on database uniqueness constraints.
- **Atomic Token Consumption**: Single-use tokens must be consumed in a single atomic SQL statement (e.g. `UPDATE magic_link SET used_at = NOW() WHERE id = $1 AND used_at IS NULL RETURNING *`).

### F. Rust Memory & Code Safety
- **Deny Unsafe**: `unsafe_code = "deny"` is enforced at the workspace level.
- **No Panic in Production**: `panic`, `unwrap()`, and `expect()` are denied by Clippy in production crates. Errors must be modeled using `DomainError` or `Result`.

---

## 2. Audit Workflow

1. **Grep for Risky Patterns**:
   ```bash
   rg "_unescaped" crates/
   rg "javascript:" crates/
   rg "redirect\(" crates/
   rg "sqlx::query" crates/
   ```
2. **Verify Route Semantics**:
   - Check that all `GET` handlers are side-effect free.
   - Check that mutation routes check session authentication and tenancy (`user.id`).
3. **Verify Error Responses**:
   - Confirm that ownership mismatches return `404` not `403`.
4. **Consult Checklist**:
   Read [Security Checklist](./references/security_checklist.md) for detailed grep heuristics and examples.
