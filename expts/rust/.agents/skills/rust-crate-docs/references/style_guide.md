# Rust Crate Documentation Style Guide

This guide establishes the conventions for documenting crates, modules, types, and functions in this workspace.

---

## 1. Doc Comment Types

- **Outer doc comments (`///`)**: Document the item that immediately follows (functions, structs, enums, traits, constants, fields).
- **Inner doc comments (`//!`)**: Document the enclosing item. Use at the top of crate roots (`src/lib.rs` or `src/main.rs`) and module files (`src/module.rs` or `src/module/mod.rs`).

---

## 2. Standard Rustdoc Sections

Document public items using standard Markdown headers:

### `# Summary`
A single line summarizing what the item does or represents. Follow with a blank line and detailed explanations if needed.

### `# Errors`
Mandatory for any function returning a `Result`. Describe:
- Each failure condition and the corresponding error variant returned.
- Example:
  ```rust
  /// # Errors
  ///
  /// Returns [`DomainError::InvalidEmail`] if the email string is malformed.
  /// Returns [`DomainError::TokenGenerationFailed`] if cryptographic RNG fails.
  ```

### `# Panics`
*(Note: Panics are strictly denied in this workspace via Clippy lints `panic = "deny"`, `unwrap_used = "deny"`, `expect_used = "deny"`).*
- If a function can panic under any edge case, it **must** be documented here.
- Public workspace APIs should prefer returning `Result` over panicking.

### `# Examples`
Provide clear, executable doctests demonstrating usage.

```rust
/// Calculates the total price for an offer.
///
/// # Examples
///
/// ```
/// use haven_domain::offer::calculate_total;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let total = calculate_total(100, 2)?;
/// assert_eq!(total, 200);
/// # Ok(())
/// # }
/// ```
```

> [!IMPORTANT]
> **Doctests and Workspace Lints**:
> Because Clippy denies `unwrap()` and `expect()`, doctests should avoid `.unwrap()` and instead return `Result<(), Box<dyn std::error::Error>>` with `?`, or use `# fn main() -> Result<...>`.

---

## 3. Intra-Doc Links

Always use intra-doc links rather than raw strings to ensure links are validated by the compiler (`RUSTDOCFLAGS="-D warnings"`):

- Structs & Types: `[`User`](crate::user::User)` or `[`DomainError`](haven_domain::DomainError)`
- Methods: `[`User::new`](crate::user::User::new)` or `[`Offer::total_cents`](crate::offer::Offer::total_cents)`
- Enum Variants: `[`DomainError::InvalidEmail`]`
- Modules: `[account module](crate::account)`

---

## 4. Architectural Layer Documentation

When documenting crates in this workspace:

1. **`haven-domain`**:
   - Focus on pure business logic, domain entities, value objects, ports (traits), and domain errors.
   - Clarify invariants (e.g. valid price ranges, non-blank titles).
2. **`haven-db`**:
   - Focus on adapter implementations of domain ports using SQLx and PostgreSQL.
   - Document transactional semantics, query behavior, and migration requirements.
3. **`haven-web`**:
   - Focus on HTTP routing, Topcoat request contexts, authentication sessions, and HTML views.
