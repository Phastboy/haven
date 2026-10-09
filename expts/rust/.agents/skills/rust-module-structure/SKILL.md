---
name: rust-module-structure
description: >-
  Decides how Rust code in this workspace (haven-domain, haven-db, haven-web,
  load-test-support) is split into files and modules, and refactors files that
  have grown too large or mixed. Use this skill when creating a new module or
  file, when a file passes the size triggers below, when asked to split,
  reorganize, or clean up a module, when adding a feature that would make an
  existing file grow, and when moving tests. Not for deciding product scope
  (use `haven-product`) or for documentation comments (use `rust-doc-authoring`).
---

# Rust module structure

## Principle

**A file groups code that changes for the same reason.** Line count is not the rule; it is a trigger to review. A 400-line file of one cohesive thing is fine. A 150-line file mixing types, validation, SQL, and rendering is not.

Rust gives freedom here: small free functions, modules of any shape, and re-exports that hide the layout from callers. Use that freedom to group by related concerns. One item per file is not the goal, and over-fragmentation is as bad as one giant file.

## Triggers (heuristics, confirm or adjust with the owner)

These are proposed starting points, not measured facts. When one fires, **review the file**; do not split automatically.

| Trigger | Action |
|---|---|
| More than about 200 lines of non-test code in a file | Review for mixed concerns |
| A `#[cfg(test)]` block longer than the code it tests, or tests that make the file past the trigger on their own | Move tests to their own file (see Tests) |
| A function longer than about 60 lines | Review; extract pure steps. (`clippy::too_many_lines` is a pedantic lint with a default of 100 lines; enable it in CI if the owner wants it enforced.) |
| A file needs several unrelated imports (database, HTTP, rendering, validation) | It is probably mixing layers |
| You must scroll to find where one concern ends and another begins | Split there |
| A change to one behaviour keeps touching the same large file as unrelated changes | Split along that change boundary |

Find candidates: `find crates -name '*.rs' -not -path '*/target/*' | xargs wc -l | sort -rn | head -20`

## How to decide where to cut

Read the file first and label every item (type, constant, function, impl, test) with its **reason to change**. Then cut along the labels. Typical axes, by layer:

| Layer | Natural axis | Example split of an `offer` area |
|---|---|---|
| `haven-domain` | by domain concept | `offer.rs` (entity and ids), `offer/price.rs`, `offer/currency.rs`, `offer/validation.rs` |
| `haven-db` | by port or operation | `offers.rs` (assembly), `offers/queries.rs`, `offers/rows.rs` (row mapping), `offers/pagination.rs` |
| `haven-web` | by route, with views separate from handlers | `offers.rs` (assembly), `offers/list.rs`, `offers/create.rs`, `offers/edit.rs`, `offers/delete.rs`, `offers/public.rs`; views in a separate `views` area |
| any | tests | `tests.rs` beside the module |

Keep related things together even if they are different kinds of item: a type and its constructor and its validation belong together if they change together. Do not split just to make files short.

## Module layout rules

1. **Parent module is assembly only.** Use the modern layout (`offers.rs` plus an `offers/` directory, no `mod.rs`). The parent file holds `mod` declarations and `pub use` re-exports, and nothing else (no logic, no types that belong in a child).
2. **Adding or improving a child touches one place.** Adding a submodule means one `mod` line (and one `pub use` if it is public API). Improving a child touches that child only. If improving one module forces edits to many others, the boundary is wrong.
3. **Preserve public paths.** Re-export from the parent so callers keep `crate::offers::Foo`. A split must not change the public API unless the owner asked.
4. **Do not widen visibility to make a split compile.** Prefer private items, then `pub(super)`, then `pub(crate)`, and `pub` only for real API. If a split needs many items made public, the cut is in the wrong place.
5. **Dependencies point one way.** Siblings do not reach into each other's internals. Shared code moves to a lower module that both depend on. No cycles.
6. **Name modules by what they are**, not by where they are used or how generic they are. Do not create `utils.rs`, `helpers.rs`, `common.rs`, `misc.rs`, or `types.rs` dumping grounds.
7. **Keep the layer boundaries:** `haven-domain` has no database or HTTP code; `haven-db` and `haven-web` depend on the domain, not the reverse. Splitting must not blur this.
8. **Handlers stay thin.** Prefer small pure functions (parse input, decide, build output) that are easy to test, composed by a thin handler that does the I/O. Pure functions go in their own module away from I/O code.

## Tests

- **Unit tests** that need private access: `#[cfg(test)] mod tests;` in the module file, with the tests in `offers/tests.rs` (or `foo/tests.rs` beside `foo.rs`). Tests then do not inflate the main file and still see private items.
- **Shared fixtures and builders** go in one `#[cfg(test)]` module (for example `test_support`), not copied per file.
- **Integration tests** (public API only, database, HTTP) go in the crate's `tests/` directory.
- **Doc tests** stay with the documentation they illustrate (see `rust-doc-authoring`).
- Do not delete, weaken, or `#[ignore]` a test to make a move easier.

## Refactor procedure

1. Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` first, so you know the starting state.
2. Read the whole file. Write down the proposed groups and the target layout. **Present it to the owner before moving code** if the split touches public paths or crosses layers.
3. **Pure move only.** No behaviour changes, renames, or "while I am here" fixes in the same change. Use `git mv` where a file keeps its identity so history follows.
4. Move one group at a time and keep the build green after each.
5. Verify the public API is unchanged (`cargo doc` paths, and the project's own callers compile without edits).
6. Re-run fmt, clippy, tests, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`.
7. Put the refactor in its own commit, separate from any logic change.

## When not to split

- The file is cohesive and just long (a table of constants, a large `match` that must stay together).
- The split would create files under about 30 lines with no concern of their own.
- The only reason is a line count.
- The code is about to be rewritten; wait.

## When to stop and ask

- You cannot name each group's single reason to change.
- The split needs public API changes or crosses a layer boundary.
- Two reasonable layouts differ by a product or architecture decision.

## Report format

```
File: <path>  (non-test lines / test lines)
Trigger(s) fired: ...
Groups found (reason to change): ...
Proposed layout: <tree>
Public API changes: none | <list>
Visibility changes: none | <list and why>
Tests: <where they move>
Verification: fmt, clippy, tests, doc results
Left unsplit on purpose: <what and why>
```
