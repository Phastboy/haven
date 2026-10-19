---
name: rust-craft
description: >-
  Standards for writing, structuring, refactoring, and documenting Rust code across
  Haven crates (haven-domain, haven-db, haven-web). Covers design principles,
  module boundaries, error handling, and Rustdoc standards.
---

# Rust code craft & architecture

## Core outcomes

1. **Easier to understand:** Clear flow, meaningful domain names, thin handlers.
2. **Easy to change:** Small cohesive files, one reason to change, explicit boundaries.
3. **Hard to break:** Compiler & lint enforcement over convention: types > lints > rules.

## 1. Code design principles

- **Main path reads straight down:** Use `?`, early returns, and `let ... else`. Edge cases and failures exit early. Top-level functions read in order without nested match pyramids.
- **Small signatures (<= 3 parameters):** If 4+ arguments are needed:
  - Check if the function does two jobs (split it).
  - Group domain fields into a dedicated validated type (`NewOffer`, `OfferEdit`).
  - Use newtypes (`OwnerId`, `OfferSlug`, `Price`) to prevent swapping parameters.
  - Never dump parameters into a generic `Params` or `Options` bag.
- **Separate pure decisions from I/O actions:**
  - *Decision:* Pure, synchronous function: `fn decide(input: DomainInput, now: Timestamp) -> Decision`. Tests require no DB, no runtime, no mocks.
  - *Action:* Thin handler/runner that performs the I/O indicated by the decision.
- **Make invalid state unrepresentable:**
  - Newtypes with private fields and validating constructors (`TryFrom`, `FromStr`). Parse at the boundary, trust the type inside.
  - Enums over boolean flags.
  - No wildcard arms on internal enums (`clippy::wildcard_enum_match_arm`): adding a variant must break compilation at every decision point.
- **External systems behind boundaries:**
  - One crate/module owns each external system (SQLx driver, mailer, Cloudinary, clock). Driver wire types and errors never appear in domain signatures.
- **Meaningful errors:**
  - Library crates (`haven-domain`, `haven-db`): `thiserror` enums with specific variants and identifying context.
  - `anyhow` is allowed only at binary edges.
  - No `.unwrap()` or `.expect()` in non-test code. Mark decisions and fallible results `#[must_use]`.

## 2. Module structure & sizing

**"A file groups code that changes for the same reason."** Line count is a review trigger, not a rigid rule.
- **Review triggers:** >200 lines of non-test code; tests longer than code; function >60 lines; mixed layers in imports.
- **Modern layout:** Parent file is assembly only (`offers.rs` alongside `offers/` directory; avoid `mod.rs`). Parent holds `mod` declarations and re-exports (`pub use`), no logic.
- **Strict layering:** `haven-domain` (pure business logic, no DB/HTTP) <- `haven-db` / `haven-web`. Dependencies point inward.
- **Visibility:** Default private -> `pub(super)` -> `pub(crate)` -> `pub`. Never widen visibility simply to make a file split compile.
- **No junk drawers:** Forbidden file names: `utils.rs`, `helpers.rs`, `common.rs`, `misc.rs`. Name files by the specific concern they own.
- **Tests:** Place unit tests requiring private access in `foo/tests.rs` with `#[cfg(test)] mod tests;` to keep source files lean.

## 3. Rustdoc & documentation

- `//!` at the top of crate/module roots describing purpose and layering position.
- `///` on all public types, traits, functions, and enum variants. Focus on **what and why**, not restating the signature.
- Sections in standard order: `# Errors` (required on functions returning `Result`), `# Panics` (state proven invariant), `# Examples`.
- **Intra-doc links:** Always link types/traits using backticks with brackets: ``[`DomainError`](crate::error::DomainError)``.
- **Doctests:** Must return `Result` and use `?`. No `.unwrap()` or `.expect()`.
- **Validation:**
  ```bash
  RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items
  cargo test --doc --workspace
  ```
