---
name: "rust-code-design-principles"
description: "Use when writing, reviewing, or refactoring Rust code: apply ten design principles with compiler- and lint-enforced mechanisms so code is readable, changeable, and hard to break."
---

# Rust code design principles

Three outcomes the code should have:

1. **Easier to understand**
2. **Easy to change**
3. **Extremely hard to break**

Prefer mechanisms the compiler or CI enforces over rules people must remember. If a rule can be a type error, make it one; if not, a deny-level lint; only then a convention or comment.

If the repo has `ENGINEERING.md`, `SECURITY.md` or `PERFORMANCE_CONTRACT.md`, those win on conflicts. For splitting files and modules use `rust-module-structure`; for doc comments use `rust-doc-authoring`. Ground every decision in facts currently visible (schema, code, requirements) and state the reason; do not build for imagined needs.

## The principles and how Rust enforces them

### 1. Keep the main path easy to follow
- Use `?`, early returns and `let ... else` so the success path reads straight down with no nested `match` pyramids.
- Edge cases and failures go in small helpers or early exits, not woven through the main flow.
- Check: can the top-level function be read in order without opening other files?

### 2. Name things by meaning
- Domain names over mechanism names: `OfferSlug`, `OwnerId`, `can_edit`, not `Data`, `handle`, `process`.
- No `utils.rs`, `helpers.rs`, `common.rs` dumping grounds; name modules for the concern they own.
- Short is fine. The name plus signature should tell a reader what it does and what it can fail with.

### 3. Keep external systems behind a boundary
Database, mail transport, payment provider, HTTP clients, the clock, the filesystem.
- One crate or module owns each external system (for example the DB layer owns the driver). Driver row types, driver errors and wire formats never appear in domain signatures.
- Translate at the boundary with `From`/`TryFrom` into domain types and domain errors.
- Inject the clock and randomness (pass `now` or a small trait) instead of calling them in logic.
- Add a trait for a boundary only when a second implementation genuinely exists (a test double counts if it removes real I/O from tests); do not speculate.
- Check: if this provider were replaced, only one module should change.

### 4. Make invalid state harder to represent
- **Newtypes with private fields** and validating constructors (`TryFrom`, `FromStr`): `Price`, `Slug`, `SessionToken`, `OfferId`. Parse once at the edge, trust the type inside.
- **Enums over flags.** Replace combinations of `bool`/`Option` that can contradict each other with an enum whose variants are exactly the valid states.
- **Typestate** for multi-step flows where order matters (unverified vs verified token), so skipping a step does not compile.
- **Deserialization is a bypass.** Do not blindly derive `Deserialize` on validated types; use `#[serde(try_from = "...")]` so the constructor runs.
- Avoid deriving `Default` on types where an empty value is not valid.
- Secrets and tokens: implement `Debug` manually (redacted) so they cannot leak into logs; no `Display` unless needed.
- **No wildcard arms on your own enums.** Write each variant out so adding a variant breaks compilation at every decision site. Enable `clippy::wildcard_enum_match_arm` for these modules.

### 5. Separate decisions from actions
- A decision is a plain, synchronous, pure function: `fn decide(input: DomainInputs, now: Timestamp) -> Decision`, where `Decision` is an enum describing what should happen. No I/O, no `async`, no hidden clock.
- An action executes the decision (write rows, send mail) and is thin.
- Tests for decisions need no database, no runtime, no mocks. Tests for actions are few and exercise the boundary.
- Check: does the rule test with plain values and `assert_eq!`?

### 6. Make errors useful
- Library crates: one `thiserror` enum per concern, a variant per distinct cause, with `#[source]` and the identifying context (which id, which field, which operation).
- `anyhow` only at the binary edge, never in domain or DB library signatures.
- Map to HTTP status and user-facing text only at the web edge. Users see a safe message; logs get the full chain. Never leak internals or secrets to users.
- No `unwrap()`/`expect()`/`panic!` in non-test code except for a proven invariant, with an `expect` message stating the invariant. Enforce with lints (below).
- Mark decisions and fallible results `#[must_use]` so they cannot be silently dropped.

### 7. Keep changes focused
- One change, one purpose; describable in a sentence without "and".
- Refactors and behaviour changes go in separate commits. A preparatory refactor lands first.
- Smaller diffs are easier to review and roll back.

### 8. Modularity and separation of concerns
- Improving one component should not mean editing the assembly (router, page layout, wiring) for each one. Assembly only composes; it holds no component logic.
- Adding a component is additive: a new module plus one registration line.
- Crate and module boundaries follow concerns (domain rules, persistence, web layer, test support), and dependencies point one way: web depends on domain and DB, domain depends on neither.
- Default to private. Widen only as far as needed: `pub(super)` when only the parent module needs it, `pub(crate)` only for items intentionally shared across the crate, and `pub` only for the crate's deliberate public API. Internals should not be reachable from outside their module unless that reach is intended.

### 9. Hard to break (outcome)
Achieved by principles 3 to 6 plus enforcement in CI:
- Workspace lints in `Cargo.toml` (`[workspace.lints]`): `unsafe_code = "forbid"`; clippy `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` at deny for non-test code; `wildcard_enum_match_arm` where decisions live.
- CI runs `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo fmt --check`.
- If the DB layer supports compile-time checked queries (for example sqlx `query!` with offline metadata), use them so a SQL typo or schema drift fails the build.
- Prefer making the wrong thing a compile error or rejected input over comments and discipline. Fail early and loudly on bad input; never continue with a guessed value.

### 10. Easy to change (outcome)
If a small requirement change forces edits in many places, a boundary or a name is in the wrong place. Fix that, not each call site.

## Onboarding rules
- **One obvious starting point:** the binary's `main` or composition root, findable in seconds.
- **Followable flow:** entry, route, decision, action, boundary, in that order, for every feature.
- **Predictable layout:** similar features are structured identically, so learning one teaches the rest.
- **No cleverness without a reason:** avoid heavy macros, deep generics and indirection that need the whole system in your head first. If something non-obvious is necessary, say why in a short comment beside it.
- **Self-describing top level:** README or `ENGINEERING.md` states what the product is, how to build, test and run, and where to start reading.
- **`cargo test` works out of the box** on a clean checkout, with any required services documented in one place.
- Check: could a newcomer find where an offer is created in under a minute?

## When reviewing or writing code
Walk the principles quickly. Report only those actually violated, each with location, principle, and a concrete Rust fix (the newtype, enum, lint or boundary to introduce). Do not pad with satisfied principles. When writing, state in one line which principle drove any non-obvious design choice.
