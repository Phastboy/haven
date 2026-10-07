---
name: rust-crate-docs
description: How to look up the real API, types, signatures, features and examples of any Rust crate used in this project, using only local sources (cargo-generated rustdoc and the crate source in the cargo registry). Use this skill BEFORE writing or changing any code that calls a third-party crate API (including Topcoat and other less-common crates), whenever a compile error mentions an unknown method, type, trait or feature, whenever the user asks "how do I do X with crate Y", and whenever you are about to rely on memory of a crate's API. If no authoritative local documentation can be found, stop and tell the user instead of guessing.
---

# Rust crate docs lookup

## Core rule

**Never write code against a crate API from memory.** Crate APIs change between versions, and the version in this project's `Cargo.lock` is the only one that matters.

Every API you use must be backed by something you actually read in this session:

- the generated rustdoc, or
- the crate's source in the cargo registry, or
- an example, test or README shipped inside the crate.

If you cannot find that evidence, **stop and tell the user** (see "When to stop"). Saying "I couldn't find docs for this" is a correct, valued outcome. Guessing, retrying variations until something compiles, or inventing method names is not.

## Project-specific notes

- This project's `ENGINEERING.md` forbids guessing APIs. Read it (and `SECURITY.md` / `PERFORMANCE_CONTRACT.md` when relevant) before using a dependency in a way that touches auth, sessions, or request handling.
- **Topcoat** is young and thinly documented elsewhere, so its guides matter more than usual. Find its manifest with the `cargo metadata` command in Step 2, then read the `docs/` folder next to it (router, view, session, cookie, context, app_context, mail). If a topic has no guide there, fall back to the source, not to memory.
- **Standard library:** use `rustup doc --std` (local), or read the std source under `$(rustc --print sysroot)/lib/rustlib/src/rust/library/` if the `rust-src` component is installed. If neither is available, say so; do not quote std APIs from memory for anything unusual.
- **Workspace crates** (`haven-domain`, `haven-db`, `haven-web`, `load-test-support`) are path dependencies: read their source directly. For writing or validating their documentation, use the `rust-doc-authoring` skill instead.

## What cargo actually keeps locally

Cargo does **not** keep rendered docs for every dependency by default. It keeps:

| What | Where | Notes |
|---|---|---|
| Crate source (`.rs`, README, CHANGELOG, `examples/`, `tests/`) | `~/.cargo/registry/src/*/<crate>-<version>/` (honours `CARGO_HOME`) | Always present after a build/fetch. Doc comments live here. |
| Rendered HTML docs | `target/doc/` | Only exists after `cargo doc` has run. |
| Git dependencies | `~/.cargo/git/checkouts/<repo>-<hash>/<rev>/` | Same idea as registry source. |
| Path dependencies / workspace crates | wherever the `path` points | Read directly. |
| Vendored crates | `vendor/` | Only if `cargo vendor` was used. |

So the workflow is: find the crate directory, then either generate rustdoc or read the source directly. Both are valid; source is the ground truth.

## Step 1: Pin down the exact crate and version

Always do this first. Do not assume the version.

```sh
# Which version is actually locked?
cargo tree -p <crate> --depth 0
grep -A2 'name = "<crate>"' Cargo.lock

# Enabled features for the crate in this build
cargo tree -p <crate> --format "{p} {f}" --depth 0

# Trace why those features are enabled
cargo tree --workspace -e features -i <crate>
```

If several versions of the crate exist in the tree (`cargo tree -d`), work out which one the project crate actually depends on before reading anything.

## Step 2: Locate the crate source directory

Prefer `cargo metadata`, because it gives the exact path for the exact resolved version, including git and path deps:

```sh
cargo metadata --format-version 1 \
  | jq -r '.packages[] | select(.name=="<crate>") | "\(.version)  \(.manifest_path)"'
```

The directory containing that `Cargo.toml` is the crate root. If `jq` is missing, fall back to:

```sh
ls -d ~/.cargo/registry/src/*/<crate>-*
```

If nothing is found, run `cargo fetch` (or `cargo build`) once and retry. If the crate is still absent, it is not a dependency of this project: tell the user.

## Step 3: Search, in this order

Work through these from cheapest and most authoritative to least. Stop as soon as you have a confident, verified answer.

1. **Project's own usage first.** `rg '<crate>::|use <crate>' --type rust` in the workspace. Existing code in this repo is the best example of the conventions in use. Also check `ENGINEERING.md` and any other project rules files for constraints on how the crate must be used.
2. **Crate README, CHANGELOG, `docs/` guides, `examples/`, `tests/`.** Examples are compiled by the crate authors and are the most reliable usage references. Check the CHANGELOG when something seems renamed or removed.
3. **Generated rustdoc** for the signature-level view (see below).
4. **Source search** in the crate root:
   ```sh
   cd <crate root>
   rg -n 'pub (async )?fn <name>' src/
   rg -n 'pub (struct|enum|trait|type|mod) <Name>' src/
   rg -n 'impl.*<Trait>.*for' src/
   rg -n '#\[cfg\(feature' src/        # is the item behind a feature flag?
   rg -n '^\s*///' src/<file>.rs        # doc comments for a file
   ```
   Read the doc comments **and** the signature, including generics, trait bounds, lifetimes, `async`, return types, and `#[deprecated]`/`#[cfg]` attributes.
5. **Cargo.toml of the crate** for the feature list: `[features]` tells you what must be enabled in this project's `Cargo.toml` for an item to exist.

Search with several different terms (the type name, the verb, the trait name, the error name) before concluding something does not exist. Re-exports are common, so an item may be defined in a private module and exposed elsewhere: search for `pub use`.

## Generating rustdoc locally

Use this when the source alone is hard to navigate (many re-exports, heavy trait machinery).

```sh
# Docs for one dependency only, no workspace crates
cargo doc -p <crate> --no-deps

# Include private items (useful when reading internals)
cargo doc -p <crate> --no-deps --document-private-items

# Docs for this project's own crates too
cargo doc --workspace --no-deps
```

Output lands in `target/doc/<crate_name>/` (hyphens become underscores). You cannot open a browser, so read it from the terminal:

```sh
# Find an item's page
ls target/doc/<crate_name>/
find target/doc/<crate_name> -name '*<Name>*.html'

# Convert a page to readable text
lynx -dump -nolist target/doc/<crate_name>/struct.<Name>.html   # if available
# otherwise
python3 - <<'EOF'
import re,sys,html
t=open("target/doc/<crate_name>/struct.<Name>.html").read()
t=re.sub(r"<(script|style).*?</\1>","",t,flags=re.S)
print(html.unescape(re.sub(r"<[^>]+>","",t)))
EOF
```

Rendered HTML is lossy and noisy. When the HTML is awkward, go back to the source files, which hold the same information.

If `cargo doc` fails (missing features, nightly-only crate), note the error and fall back to reading the source. Do not change the project's toolchain or features just to build docs without asking.

## Let the compiler confirm, but not guide

- After writing code, run `cargo check` (add `-p <crate>` for the project crate) to verify it.
- Compiler errors and "help:" suggestions are evidence. Read them carefully, then confirm the suggested item in the docs/source before applying it.
- If you have gone through **two** failed edit-and-check rounds on the same API problem, stop. Do not continue cycling through variations. Go back to Step 3 and re-read the source, or report to the user.

## When to stop and tell the user

Stop and report, without writing speculative code, when any of these is true:

- The crate is not in `Cargo.lock`, or its source is not present locally.
- You cannot find the item (function, type, trait, feature) after searching the source with several terms.
- The docs and source disagree with what the user asked for (for example, the method exists only in a newer version or behind a feature the project has not enabled).
- Two or more plausible APIs exist and the docs do not say which is intended.
- The crate is undocumented (no doc comments, no examples, no README guidance) for the thing being asked.
- `cargo doc` cannot be generated and the source is not enough to be sure.

Report in this shape:

```
Could not verify: <what you were trying to do / which API>
Crate and version: <name> <version from Cargo.lock>
Searched: <paths, commands, search terms>
Found: <what exists that is closest, if anything>
Needed from you: <link to docs, a version bump, a feature flag, or a decision>
```

Never fill the gap with a plausible-looking guess. Do not present unverified code as working.

## Reporting what you used

When you give an answer or finish a change involving a crate API, include a short evidence line, for example:

```
Verified against: <crate> 1.2.3, src/client.rs:88 (pub async fn send), examples/basic.rs
```

Keep it to the file and line you actually read. If part of the answer is unverified, say which part.

## Quick reference

```sh
cargo tree -p <crate> --depth 0                      # locked version
cargo tree -p <crate> --format "{p} {f}" --depth 0   # enabled features
cargo tree --workspace -e features -i <crate>        # trace why features are enabled
cargo tree -d                                        # duplicate versions
cargo metadata --format-version 1 | jq -r '.packages[]|select(.name=="<crate>")|.manifest_path'
cargo doc -p <crate> --no-deps                       # build rustdoc
cargo check                                          # confirm it compiles
rg -n 'pub fn <name>' <crate root>/src               # find a function
rg -n 'pub use' <crate root>/src                     # follow re-exports
```
