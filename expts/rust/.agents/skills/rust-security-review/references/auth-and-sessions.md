# Authentication, sessions, and CSRF

Scope: login, registration, password reset, sessions, cookies, and any route that changes state.

**Before judging any of this, verify what Topcoat's session and cookie support actually does** (signing, encryption, default flags, expiry, rotation) using `rust-crate-docs`. Do not assume secure defaults.

## Broken authentication

Check:

**Passwords**
- [ ] Hashed with a modern password hash (argon2id preferred; bcrypt or scrypt acceptable), with per-password random salt. Never plain, never a fast hash (SHA-256, MD5).
- [ ] Hashing runs off the async executor (`spawn_blocking`) and has cost parameters that fit the instance's CPU and memory. Record the chosen parameters.
- [ ] Password comparison uses the hash library's verify function (constant time).
- [ ] Minimum length enforced; no composition rules that force weak patterns; maximum length bounded (prevents hashing DoS).
- [ ] Passwords and hashes are never logged, never returned in responses, never in error messages.

**Login flow**
- [ ] Same error message and similar timing for "unknown user" and "wrong password" (no user enumeration). Run a dummy hash verify for unknown users.
- [ ] Rate limiting or lockout/backoff on login, registration, password reset, and any code-verification endpoint.
- [ ] Registration and reset responses do not reveal whether an email exists.
- [ ] Password reset tokens are random (CSPRNG), single-use, short-lived, stored hashed, and invalidated after use or after a password change.

**Sessions**
- [ ] Session ids come from a CSPRNG with enough entropy (at least 128 bits).
- [ ] **A new session id is issued at login** (prevents session fixation) and on privilege change.
- [ ] Logout invalidates the session server-side, not just by clearing the cookie.
- [ ] Idle and absolute expiry exist and are enforced server-side.
- [ ] Password change or reset invalidates other sessions.
- [ ] Session storage failures fail closed (deny), not open.

**Cookies**
- [ ] `HttpOnly` (blocks script access), `Secure` (HTTPS only), explicit `SameSite` (`Lax` at least; `Strict` where it fits).
- [ ] Reasonable `Path`/`Domain`; prefer the `__Host-` prefix when possible (requires `Secure`, `Path=/`, no `Domain`).
- [ ] No sensitive data in the cookie value besides an opaque id (or properly encrypted and authenticated data, if the framework does that; verify).

**Route protection**
- [ ] Authentication is enforced by a central mechanism (extractor, middleware, guard), not by each handler remembering to check. Look for routes that **forgot** it: list all routes, mark which require auth, compare.
- [ ] Default is deny: a new route is protected unless explicitly marked public.
- [ ] Authorization (what this user may do) is separate from authentication and enforced for every action. See IDOR in `data-access.md`.
- [ ] Secrets (signing keys, session secrets) come from env/secret storage, are long and random, and are not committed. Rotation is possible.

**Out of scope unless asked:** MFA, OAuth/OIDC flows. If present, review state/nonce/PKCE handling and redirect URI validation.

## CSRF

**Threat:** a malicious site makes the victim's browser send an authenticated request (cookies attach automatically).

Check:
- [ ] **State-changing actions never use GET** (or HEAD). Logout, delete, follow, like, etc. are POST/PUT/PATCH/DELETE.
- [ ] Cookie `SameSite` is `Lax` or `Strict`. This blocks most cross-site POSTs but is **defense in depth, not the only defense** (subdomain and same-site attacks, older browsers).
- [ ] State-changing routes verify **an anti-CSRF token** (synchronizer token tied to the session, or signed double-submit), or the framework provides an equivalent that you verified in its source.
- [ ] Additionally validate the `Origin` header (fallback to `Referer`) against the expected origin for state-changing requests.
- [ ] Token comparison is constant time, tokens are unpredictable and session-bound.
- [ ] Forms include the token; fetch/XHR requests send it in a header; the token is not in the URL.
- [ ] JSON endpoints are not CSRF-safe merely because they expect JSON. Require a custom header or check `Origin`, and do not accept form content types on JSON routes.
- [ ] CORS is not configured as `*` with credentials; allowed origins are an explicit list. CORS is not a CSRF defense for simple requests.
- [ ] Login CSRF considered (an attacker logging the victim into the attacker's account).

If the framework has no built-in CSRF protection (verify!), report it as a finding and propose the smallest middleware that does token plus Origin checks.

Test: a request to each state-changing route with a valid session cookie but **no or wrong token** and a foreign `Origin` must be rejected.
