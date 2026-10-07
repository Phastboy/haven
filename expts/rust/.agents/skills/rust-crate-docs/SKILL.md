---
name: rust-crate-docs
description: >-
  Generates, verifies, inspects, and standardizes Rustdoc documentation for crates
  in this workspace (haven-domain, haven-db, haven-web, load-test-support) as well
  as third-party dependencies. Use this skill when asked to document Rust code,
  write rustdoc comments, run doctests, check for broken documentation links, or
  consult offline dependency API documentation.
---

# Rust Crate Documentation Skill (`rust-crate-docs`)

This skill governs generating, validating, and authoring Rustdoc documentation for the workspace crates and inspecting dependency APIs without guessing.

---

## 1. Workspace Crates Overview

The workspace contains the following crates:
- **[`haven-domain`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain)**: Pure domain entities, value objects, domain errors, and port traits.
- **[`haven-db`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-db)**: PostgreSQL database adapters, SQLx queries, and port implementations.
- **[`haven-web`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web)**: Web server application built on Topcoat, route handlers, HTML views, and sessions.
- **[`load-test-support`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/load-test-support)**: Utilities and harness for load testing.

---

## 2. Core Workflows

### A. Build Workspace Docs
Generate documentation for all workspace crates (including private items for internal documentation):
```bash
cargo doc --workspace --no-deps --document-private-items
```
The output HTML is located at `target/doc/<crate_name>/index.html`.

### B. Strict Documentation Validation (Zero Warnings)
Verify that there are no broken intra-doc links, missing doc attributes, or syntax errors:
```bash
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items
```
Or execute the helper script:
```bash
./.agents/skills/rust-crate-docs/scripts/check_docs.sh
```

### C. Run Doctests
Execute all doc tests across the workspace:
```bash
cargo test --doc --workspace
```
*Note: Ensure doc tests follow workspace lint rules (avoid `.unwrap()`; use `?` and return `Result`).*

---

## 3. Mandatory Dependency Documentation Rule

Per [`ENGINEERING.md`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/ENGINEERING.md), **never guess an API**. Before writing or refactoring code that uses external dependencies:

1. **Topcoat Guides & Manifest**:
   Locate Topcoat source and read the guide docs:
   ```bash
   cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="topcoat") | .manifest_path'
   ```
   Read the `docs/` folder beside that manifest (router, view, session, cookie, context, app_context, mail).
2. **Exact Signatures**:
   Generate or view doc comments for the dependency:
   ```bash
   cargo doc -p topcoat --no-deps
   cargo doc -p sqlx --no-deps
   ```
3. **Rust Standard Library**:
   Consult local std docs via:
   ```bash
   rustup doc --std
   ```
   or browse online at `https://doc.rust-lang.org/std/`.

---

## 4. Documentation Standards & Style

When writing documentation in any crate:
- Always follow the conventions in [Style Guide](./references/style_guide.md).
- Use `//!` at the top of files for module and crate-level overviews.
- Use `///` for public types, traits, functions, and enum variants.
- Include `# Errors` sections on all functions returning `Result`.
- Include `# Panics` sections (though panics should generally be avoided per Clippy lints).
- Use intra-doc links (e.g. `[`DomainError`](crate::error::DomainError)`).
