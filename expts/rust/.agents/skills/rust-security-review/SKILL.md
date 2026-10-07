---
name: rust-security-review
description: >-
  Reviews and hardens Rust web code in this workspace (haven-domain, haven-db,
  haven-web) for security and performance problems: N+1 queries, SQL injection,
  broken authentication, SSRF, XSS, CSRF, IDOR, security misconfiguration,
  vulnerable components, HSTS and CSP. Use this skill when asked to audit,
  review, or harden code; when adding or changing routes, handlers, queries,
  sessions, cookies, forms, views, outbound HTTP calls, response headers, or
  dependencies; and before declaring a feature done if it touches user input or
  authorization. Do not use it for general documentation or dependency API
  lookup; use `rust-doc-authoring` and `rust-crate-docs` for those.
---

# Rust security and performance review

## Core rules

1. **Evidence, not assumption.** Every finding must point to a file and line you actually read. Every "this is safe" claim must be backed by code or docs you read. Never say the framework "handles it" unless you verified that in the code or docs.
2. **Framework behaviour is verified with `rust-crate-docs`.** Topcoat, SQLx and any other dependency may or may not escape output, protect against CSRF, set headers, or sign cookies. Look it up in the locked version's source and docs. If you cannot confirm it, report it as **unverified** and say what you checked.
3. **Read the project's own rules first:** `ENGINEERING.md`, `SECURITY.md`, `PERFORMANCE_CONTRACT.md`. Findings that contradict them are higher severity. Do not claim a guarantee those files or the code do not support.
4. **Review mode is read-only by default.** Report findings; change code only when asked. Never "fix" by weakening a check, adding `#[allow]`, or removing a test.
5. **Every fix gets a regression test** where one is practical (a unit test, integration test, or a k6 authorization case). A fix without a test can silently regress.
6. **If you cannot verify something, say so and stop.** An honest "unverified" is better than a confident guess.

## How to run a review

1. Identify the scope: which crates, routes, queries, or dependencies changed or are in question.
2. Map the **attack surface** in the scope: every place user input enters (path, query, form, header, cookie, body), every query, every outbound request, every response written.
3. Load the reference file(s) for the topics that apply (table below). Load only what you need.
4. Trace each input from entry to sink. Check each item in the relevant checklist against the actual code.
5. Report using the format at the end.

## Which reference to load

| Touching... | Read |
|---|---|
| SQL, repositories, `haven-db`, loops that fetch data, authorization of records | `references/data-access.md` (SQL injection, N+1, IDOR, other query performance) |
| Login, sessions, cookies, forms, state-changing routes | `references/auth-and-sessions.md` (broken authentication, CSRF) |
| Views, templates, user-generated content, response headers, TLS | `references/output-and-headers.md` (XSS, CSP, HSTS, misconfiguration) |
| Outbound HTTP, URL/webhook/import features, `Cargo.toml`, CI, Docker | `references/outbound-and-dependencies.md` (SSRF, component vulnerabilities) |

A full audit loads all four.

## What belongs in tooling, not just in this skill

This skill guides review. Anything that can be enforced mechanically should also be enforced in CI (see `CICD_PLAN.md`):

- `cargo clippy ... -D warnings`, with lints such as `clippy::unwrap_used`, `clippy::expect_used`, `clippy::indexing_slicing`.
- `#![forbid(unsafe_code)]` in crates that do not need `unsafe`.
- `cargo deny check` and Dependabot (advisories, licenses, duplicates).
- Authorization and ownership tests (k6 `04` authorization script and integration tests).
- Trivy scan on the built image.

If a finding could be caught by one of these, recommend adding the check, not only fixing the instance.

## Severity

| Level | Meaning |
|---|---|
| Critical | Exploitable now, by an unauthenticated or low-privilege user, with serious impact (auth bypass, SQL injection, cross-user data access) |
| High | Exploitable with some precondition, or serious impact if the framework assumption is wrong |
| Medium | Defense-in-depth gap or performance problem that violates the performance contract |
| Low | Hardening or hygiene |
| Unverified | Could not confirm either way; state what is needed to confirm |

## Report format

```
[Severity] Short title
Where: path/to/file.rs:LINE
What: what the code does, in one or two sentences
Why it matters: concrete attack or performance impact
Evidence: what you read (code lines, docs, crate version)
Fix: the smallest correct change
Test: the regression test to add
```

End with:
- **Checked and fine:** what you verified is safe, with evidence.
- **Unverified:** what you could not confirm and what you need.
- **Not reviewed:** scope you did not cover.
