---
name: rust-security-and-deps
description: >-
  Guides dependency verification and security/performance review in Haven
  (haven-domain, haven-db, haven-web). Covers local crate docs lookup (Topcoat, SQLx),
  OWASP web vulnerabilities (IDOR, CSRF, XSS, SSRF, SQLi), query efficiency (N+1),
  and auth. Use before writing code against 3rd-party crates or auditing handlers/queries.
---

# Rust security & dependency verification

## 1. Third-party crate documentation rule

**Never write code against a third-party crate API from memory.** Crate APIs change between versions; only the exact version in `Cargo.lock` matters.

### Ground truth workflow
1. **Identify locked version & features:**
   ```bash
   cargo tree -p <crate> --depth 0
   grep -A2 'name = "<crate>"' Cargo.lock
   ```
2. **Locate crate source:**
   ```bash
   cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="<crate>") | "\(.version)  \(.manifest_path)"'
   # or fallback:
   ls -d ~/.cargo/registry/src/*/<crate>-*
   ```
3. **Search order:**
   - Existing usage in workspace (`rg '<crate>::' crates/`).
   - Crate's local README, `examples/`, `tests/`.
   - **Topcoat:** Read its local `docs/` folder (router, view, session, cookie, context, app_context, mail) next to its source manifest.
   - Crate source `src/`: inspect function signatures, doc comments, and `#[cfg(feature = "...")]` flags.
4. **When to stop:** If you cannot find documentation or source evidence for an API, stop and inform the user instead of guessing variations.

## 2. Security & performance review checklist

Every finding must cite evidence (file, line number, or crate doc). Never assume framework protections without verification.

### Data access & SQL
- **SQL injection:** Use SQLx parameterized queries exclusively. Never interpolate strings into SQL.
- **N+1 queries:** Prohibit executing queries inside loops over result sets. Use `JOIN` or batched `WHERE id = ANY(...)`.
- **IDOR (Ownership scoping):** Every read/write to non-public records must verify that the requesting session owns the record (`WHERE id = $1 AND owner_id = $2`).

### Auth, sessions, and forms
- **Broken authentication:** Verify session token validity and expiration on every protected route. Use constant-time comparisons for tokens/hashes.
- **CSRF protection:** Every state-changing form (POST/PUT/DELETE) must validate an anti-CSRF token.
- **Secret redaction:** Implement `Debug` manually for sensitive types (passwords, tokens, cookies) to prevent leaking to logs.

### Output, headers, and views
- **XSS prevention:** Ensure all user input rendered into HTML is escaped by Topcoat. Never bypass escaping for user content.
- **Headers:** Enforce strict CSP (`default-src 'self'`), `Referrer-Policy: same-origin`, and HSTS headers.

### Outbound requests & uploads
- **SSRF:** Validate external URLs before fetching; restrict to allowed protocols and block private/loopback IP ranges.
- **File uploads:** Validate file magic bytes on server (JPEG/PNG/WebP only; **strictly no SVG**). Strip EXIF location metadata before storing.

## 3. Severity & reporting format

- **Critical:** Unauthenticated/low-privilege exploit (auth bypass, SQLi, cross-account IDOR).
- **High:** Exploitable with preconditions or unverified framework assumption.
- **Medium:** Defense-in-depth gap or performance violation (`PERFORMANCE_CONTRACT.md`).
- **Low:** Hygiene or minor hardening.
- **Unverified:** Could not confirm either way; state what investigation is needed.

### Report format
```
[Severity] Short title
Where: crates/path/to/file.rs:LINE
What: what the code does
Why it matters: attack scenario or performance impact
Evidence: verified code lines or crate source
Fix: minimal correct change
Test: regression test to add
```
