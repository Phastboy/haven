---
name: rust-doc-authoring
description: >-
  Writes, reviews, and validates Rustdoc documentation for the crates in this
  workspace (haven-domain, haven-db, haven-web, load-test-support). Use this skill
  whenever asked to document Rust code, add or fix `///` or `//!` comments, run
  doctests, fix broken intra-doc links, or check that `cargo doc` is warning-free.
  Not for looking up how a third-party dependency's API works; use the
  `rust-crate-docs` skill for that.
---

# Rust documentation authoring

## Core rules

1. **Document only what the code actually does.** Read the function body, its callers, and the types involved before writing a doc comment. Do not describe behaviour you have not verified.
2. **If you cannot tell what something is for, ask the user.** Do not invent intent, invariants, or guarantees. A missing doc comment is better than a wrong one.
3. **Do not touch code logic while documenting.** Documentation changes only. If you spot a bug or a wrong name, report it separately.
4. **Respect `ENGINEERING.md`.** Doctests follow workspace lint rules: no `.unwrap()` / `.expect()`; use `?` and return `Result`.

## Workspace crates

Paths are relative to the Rust workspace root (the directory containing the workspace `Cargo.toml`).

| Crate | Path | Purpose |
|---|---|---|
| `haven-domain` | `crates/haven-domain` | Pure domain entities, value objects, domain errors, port traits |
| `haven-db` | `crates/haven-db` | PostgreSQL adapters, SQLx queries, port implementations |
| `haven-web` | `crates/haven-web` | Topcoat web app: route handlers, HTML views, sessions |
| `load-test-support` | `crates/load-test-support` | Load-test seeding and harness utilities |

Layering matters for docs: `haven-domain` must not mention database or HTTP details; `haven-db` and `haven-web` link *to* domain types, not the other way round.

## Commands

Run from the workspace root.

```bash
# Build docs for workspace crates, including private items
cargo doc --workspace --no-deps --document-private-items

# Strict mode: any rustdoc warning (broken link, bad syntax) fails
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items

# Doctests
cargo test --doc --workspace

# One crate only (faster while iterating)
cargo doc -p haven-domain --no-deps --document-private-items
cargo test --doc -p haven-domain
```

Output is in `target/doc/<crate_name>/index.html` (hyphens become underscores). If `.agents/skills/rust-doc-authoring/scripts/check_docs.sh` exists, it may be used as a shortcut for the strict build, but the commands above are the source of truth.

## Workflow

1. Read the item and its surrounding module first.
2. Write or edit the doc comments.
3. Run the strict `cargo doc` command for the crate you touched. Fix every warning.
4. Run `cargo test --doc -p <crate>`. Fix every failure.
5. Report what you documented, and anything you left undocumented because intent was unclear.

If a command fails for a reason unrelated to your doc changes (existing compile error, missing database, etc.), report it and stop. Do not "fix" unrelated code to make docs build.

## Style

For detailed conventions and formatting rules, see [Style Guide](./references/style_guide.md).

- `//!` at the top of a file or `lib.rs` for module and crate overviews: what it is for, and where it sits in the layering.
- `///` on every public type, trait, function, and enum variant. Document private items when their reason for existing is not obvious.
- First line: one short sentence in the third person that stands alone (it appears in search results and module listings). Then a blank line, then details.
- Describe **what and why**, not a restatement of the signature. Do not write "Returns the id" for `fn id(&self) -> Id`.
- Sections, in this order when present: `# Errors`, `# Panics`, `# Safety`, `# Examples`.
  - `# Errors`: required on every function returning `Result`. List each error condition, linking the error type.
  - `# Panics`: only when a panic is possible. Panics should be rare in this project; if you find one, mention it to the user.
  - `# Examples`: for public APIs where usage is not obvious. Keep them short and make them compile.
- **Intra-doc links** for every type, trait, and function you mention: `` [`DomainError`] `` or `` [`DomainError`](crate::error::DomainError) ``. Do not use plain backticks for items that can be linked. Rustdoc verifies links; plain text does not get checked.
- Security-relevant and performance-relevant behaviour (auth requirements, owner scoping, idempotency, rate limiting) must be stated where it applies. Check `SECURITY.md` and `PERFORMANCE_CONTRACT.md` before documenting such behaviour, and do not claim a guarantee that those files or the code do not support.
- Doc examples that cannot run (need a database, network, or a running server) use `no_run`, never `ignore`, unless there is a reason you state in the comment. Never hide a failing example behind `ignore`.

## Doctest rules

- Return `Result` and use `?`. Use the hidden-line form for plumbing:

  ```rust
  /// ```
  /// # use haven_domain::Example;
  /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
  /// let value = Example::new("a")?;
  /// # Ok(())
  /// # }
  /// ```
  ```

- No `.unwrap()` or `.expect()` in examples, even though it is common in the wild.
- Examples must only use public API of the crate being documented.

## When to stop and ask

- You do not understand why an item exists or what an invariant is.
- The doc would need to describe behaviour of a dependency you have not verified. Use `rust-crate-docs` first.
- A warning cannot be fixed without changing code.
- The strict doc build or doctests fail for reasons you do not understand.

Report what you checked and what you need. Do not paper over the gap with plausible wording.
