# Output, headers, and configuration: XSS, CSP, HSTS, misconfiguration

Scope: HTML views, anything that renders user-controlled data, response headers, TLS behaviour, and runtime configuration.

**Verify what Topcoat's view layer escapes and how it sets headers** using `rust-crate-docs` before concluding anything. Find the escape function and any "raw"/"unescaped" escape hatch, and search the repo for every use of it.

## XSS (cross-site scripting)

**Rule:** user-controlled data is escaped for the exact context it is placed in.

Check:
- [ ] Default view output is HTML-escaped (verified in the framework source, not assumed).
- [ ] Every use of a raw/unescaped/"trusted HTML" mechanism is listed and justified. Search the repo for it. Each one must receive data that is sanitized or fully static.
- [ ] **Context matters.** HTML text escaping does not protect these contexts:
  - Attribute values (always quoted; no user data in event handler attributes like `onclick`).
  - URL attributes (`href`, `src`, `action`): reject `javascript:`, `data:` and other unexpected schemes; allow only `http`, `https`, `mailto` (or a stricter list) for user-supplied URLs.
  - Inline `<script>` and JSON embedded in HTML: do not interpolate user data; if data must be passed, serialize as JSON and escape `<`, `>`, `&`, U+2028/2029, or use a data attribute.
  - Inline `<style>` and `style=""` attributes: no user data.
- [ ] User-supplied rich text/Markdown is rendered through a sanitizer with an allowlist, not by escaping-then-trusting.
- [ ] Error pages, flash messages, and validation messages that echo input are escaped too.
- [ ] Responses set correct `Content-Type` with charset; user-uploaded files are served with a safe type and `X-Content-Type-Options: nosniff`, preferably from a separate origin and with `Content-Disposition: attachment` where inline display is not needed.
- [ ] JSON responses use `application/json` (never `text/html`).
- [ ] Client-side JS (if any) avoids `innerHTML`, `document.write`, `eval` with user data.

## CSP (Content Security Policy)

CSP is **defense in depth** for XSS. It does not replace escaping.

Recommended starting point (adjust to what the app actually loads):

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self';
  style-src 'self';
  img-src 'self' data:;
  font-src 'self';
  connect-src 'self';
  object-src 'none';
  base-uri 'none';
  form-action 'self';
  frame-ancestors 'none'
```

Check:
- [ ] A CSP header is sent on HTML responses. Decide the policy from what the pages **actually load** (inspect the views), not from a template.
- [ ] No `'unsafe-inline'` or `'unsafe-eval'` for scripts. If inline scripts are needed, use per-request **nonces** (random, unguessable, new for every response) or hashes.
- [ ] `object-src 'none'`, `base-uri` restricted, `form-action` restricted, `frame-ancestors` set (clickjacking; also set `X-Frame-Options: DENY` for older browsers if desired).
- [ ] Third-party origins (CDNs, analytics, fonts) are listed explicitly and minimal. Prefer self-hosting assets.
- [ ] Roll out with `Content-Security-Policy-Report-Only` first, fix violations, then enforce.
- [ ] A test asserts the header is present on an HTML route.

## HSTS (HTTP Strict Transport Security)

```
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

Check:
- [ ] Sent **only over HTTPS** responses (browsers ignore it over HTTP).
- [ ] `max-age` at least 6 months, ideally one year, after a short trial period with a small value if unsure.
- [ ] `includeSubDomains` only if every subdomain supports HTTPS.
- [ ] **Do not add `preload` unless you are certain.** Preload submission is hard to undo; it is a deliberate, one-way decision for the whole domain.
- [ ] **Where TLS terminates matters.** If a platform proxy or edge terminates TLS in front of the app (likely on a managed host such as Koyeb, but verify in their docs), decide whether the proxy or the app sets HSTS, and check the app sees the original scheme (via a trusted forwarded header) so it does not mistake HTTPS requests for HTTP. Only trust forwarded headers from the proxy, never from arbitrary clients.
- [ ] HTTP-to-HTTPS redirect exists where the platform does not do it already.
- [ ] Cookies are `Secure` (see `auth-and-sessions.md`).

## Security misconfiguration

Check:
- [ ] Other security headers on all responses: `X-Content-Type-Options: nosniff`, `Referrer-Policy` (for example `strict-origin-when-cross-origin` or `no-referrer`), `Permissions-Policy` limiting unused features, `Cross-Origin-Opener-Policy` where useful. Apply them in one central place (middleware/layer), not per handler.
- [ ] **No debug features in production:** debug routes, verbose error pages, stack traces, panic messages, SQL errors, or internal paths in responses. Errors return generic messages; details go to logs.
- [ ] Logs do not contain passwords, tokens, session ids, full cookies, `DATABASE_URL`, or personal data beyond what is necessary.
- [ ] Secrets only from environment or secret store; none in the repo, in `Dockerfile`, build args, image layers, or CI logs. Check `git log` for accidentally committed secrets if there is any doubt.
- [ ] CORS: explicit origin allowlist; no wildcard with credentials.
- [ ] Request limits exist: body size, header size, timeouts, and concurrency limits, so one client cannot exhaust the server.
- [ ] Unused routes, methods (`TRACE`), and example/test endpoints are removed. Admin and metrics routes are not publicly reachable.
- [ ] Default accounts or seeded test users (including the load-test seed) cannot exist in production. Seeding code must be unreachable or disabled there.
- [ ] Database user has least privilege (no superuser, no schema-owner rights for the app's runtime role if avoidable).
- [ ] Container: runs as non-root, minimal image (`scratch`), read-only filesystem if possible, no unnecessary capabilities, only the needed port exposed.
- [ ] Health route reveals nothing sensitive (no versions, no dependency details).
- [ ] Production and development configuration are separated; production fails to start if required security configuration is missing (fail closed).
