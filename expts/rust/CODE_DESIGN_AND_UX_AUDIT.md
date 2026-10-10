# Haven Code Design & UX Audit

> **Audit Date:** 2026-10-10  
> **Audited Skills:** `rust-code-design-principles` and `haven-ux-design`  
> **Workspace Scope:** `crates/haven-domain`, `crates/haven-db`, `crates/haven-web`, `load-test-support`, root configuration (`Cargo.toml`), and design assets (`crates/haven-web/static/style.css`).  
> **Methodology:** Per the audit rules of both skills, **only violations are reported**. Conforming areas are not padded. Each item contains exact locations, the principle/rule violated, and a concrete fix.

---

## Table of Contents
1. [Rust Code Design Principles Violations](#1-rust-code-design-principles-violations)
   - [Principle 3 & 5: External Boundaries & Clock Injection](#11-principle-3--5-external-boundaries--clock-injection)
   - [Principle 4: Making Invalid State Harder to Represent](#12-principle-4-making-invalid-state-harder-to-represent)
   - [Principle 6: Making Errors Useful](#13-principle-6-making-errors-useful)
   - [Principle 8: Modularity & Visibility Ladder](#14-principle-8-modularity--visibility-ladder)
   - [Principle 9: Hard to Break & Workspace Lints](#15-principle-9-hard-to-break--workspace-lints)
2. [Haven UX Design Violations](#2-haven-ux-design-violations)
   - [Section 1 & 6: Destructive Deletion Without Confirmation](#21-section-1--6-destructive-deletion-without-confirmation)
   - [Section 1 & 6: Form Validation Failures Drop to HTTP 400](#22-section-1--6-form-validation-failures-drop-to-http-400)
   - [Section 1: Static Header Missing User State & Sign-Out Navigation](#23-section-1-static-header-missing-user-state--sign-out-navigation)
   - [Section 4 & 6: Missing `:active` (Pressed) Interaction States](#24-section-4--6-missing-active-pressed-interaction-states)
   - [Section 7 & DESIGN.md: Non-Greyscale Color Tokens](#25-section-7--designmd-non-greyscale-color-tokens)
   - [Section 6 & Accessibility: Inputs Lack ARIA Error Association](#26-section-6--accessibility-inputs-lack-aria-error-association)
3. [Prioritized Remediation Roadmap](#3-prioritized-remediation-roadmap)

---

## 1. Rust Code Design Principles Violations

### 1.1 Principle 3 & 5: External Boundaries & Clock Injection
*Principle 3:* "Inject the clock and randomness (pass `now` or a small trait) instead of calling them in logic."  
*Principle 5:* "A decision is a plain, synchronous, pure function: `fn decide(input: DomainInputs, now: Timestamp) -> Decision`... No I/O, no async, no hidden clock."

- **Location:** [`crates/haven-domain/src/session.rs:121-123`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/session.rs#L121-L123)
  ```rust
  pub fn is_valid(&self) -> bool {
      Utc::now() < self.expires_at
  }
  ```
- **Location:** [`crates/haven-domain/src/magic_link.rs:25-27`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/magic_link.rs#L25-L27)
  ```rust
  pub fn is_valid(&self) -> bool {
      self.used_at.is_none() && Utc::now() < self.expires_at
  }
  ```
- **Violation:** The domain layer calls the real system clock `Utc::now()` directly inside core entity methods, making validation impure, non-deterministic, and impossible to unit test against arbitrary timestamps without sleeping.
- **Concrete Rust Fix:** Replace parameterless `is_valid()` with a pure, clock-injected signature:
  ```rust
  pub fn is_valid_at(&self, now: DateTime<Utc>) -> bool {
      now < self.expires_at
  }
  ```
  Callers at the application boundary (web handlers or services) inject `Utc::now()`.

---

### 1.2 Principle 4: Making Invalid State Harder to Represent
*Principle 4:* "Newtypes with private fields and validating constructors (`TryFrom`, `FromStr`)... Deserialization is a bypass. Do not blindly derive `Deserialize` on validated types; use `#[serde(try_from = "...")]`... Avoid deriving `Default` on types where an empty value is not valid... Enums over flags."

#### 1.2.1 Public Inner Fields on Entity ID Newtypes
- **Locations:**
  - [`crates/haven-domain/src/offer.rs:31`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/offer.rs#L31): `pub struct OfferId(pub Uuid);`
  - [`crates/haven-domain/src/user.rs:12`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/user.rs#L12): `pub struct UserId(pub Uuid);`
  - [`crates/haven-domain/src/account.rs:10`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/account.rs#L10): `pub struct AccountId(pub Uuid);`
  - [`crates/haven-domain/src/session.rs:13`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/session.rs#L13): `pub struct SessionId(pub Uuid);`
  - [`crates/haven-domain/src/ports.rs:16`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/ports.rs#L16): `pub struct IdempotencyKey(pub Uuid);`
  - [`crates/haven-domain/src/magic_link.rs:15`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/magic_link.rs#L15): `pub id: Uuid;` (unwrapped primitive `Uuid` instead of a typed ID).
- **Violation:** Public inner fields allow external callers to construct IDs arbitrarily (`OfferId(uuid)`), bypassing constructors. `MagicLink` lacks an ID newtype altogether.
- **Concrete Rust Fix:**
  - Make inner fields private: `pub struct OfferId(Uuid);`.
  - Provide `OfferId::from_uuid(uuid: Uuid) -> Self` or `From<Uuid> / TryFrom<Uuid>` and `as_uuid(&self) -> Uuid`.
  - Introduce `pub struct MagicLinkId(Uuid);` in `haven-domain::magic_link`.

#### 1.2.2 Non-Deterministic Random Generation in `Default` Implementations
- **Locations:**
  - [`crates/haven-domain/src/offer.rs:47-51`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/offer.rs#L47-L51): `impl Default for OfferId { fn default() -> Self { Self::new() } }`
  - [`crates/haven-domain/src/user.rs:28-32`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/user.rs#L28-L32): `impl Default for UserId { fn default() -> Self { Self::new() } }`
  - [`crates/haven-domain/src/account.rs:22-26`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/account.rs#L22-L26): `impl Default for AccountId { fn default() -> Self { Self::new() } }`
  - [`crates/haven-domain/src/session.rs:21-25`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/session.rs#L21-L25): `impl Default for SessionId { fn default() -> Self { Self::new() } }`
- **Violation:** `Default::default()` is expected to return an empty, zero, or standard initial state without side effects. Generating random v4 UUIDs inside `default()` introduces impure side effects where a default value does not conceptually exist.
- **Concrete Rust Fix:** Remove `Default` implementations for entity ID types. Callers must explicitly call `OfferId::new()` or pass a known identifier.

#### 1.2.3 Deserialization Bypassing Validating Constructors
- **Locations:**
  - [`crates/haven-domain/src/offer/price.rs:8`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/offer/price.rs#L8): `#[derive(..., Deserialize)] pub struct Price(i32);`
  - [`crates/haven-domain/src/offer/slug.rs:18`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/offer/slug.rs#L18): `#[derive(..., Deserialize)] pub struct OfferSlug(String);`
  - [`crates/haven-domain/src/account.rs:36`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/account.rs#L36): `#[derive(..., Deserialize)] pub struct Email(String);`
  - [`crates/haven-domain/src/offer/currency.rs:11`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/offer/currency.rs#L11): `#[derive(..., Deserialize)] pub struct CurrencyCode(String);`
- **Violation:** Directly deriving `Deserialize` bypasses `Price::new`, `OfferSlug::parse`, `Email::parse`, and `CurrencyCode::parse`. A JSON payload with `{"price": -50}` or `{"email": "notanemail"}` deserializes into domain structs without validation errors.
- **Concrete Rust Fix:** Use Serde's `try_from` attribute to route deserialization through validating constructors:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(try_from = "i32", into = "i32")]
  pub struct Price(i32);

  impl TryFrom<i32> for Price {
      type Error = DomainError;
      fn try_from(val: i32) -> Result<Self, Self::Error> {
          Self::new(val)
      }
  }
  ```
  Apply identically to `OfferSlug` (`try_from = "String"`), `Email` (`try_from = "String"`), and `CurrencyCode`.

#### 1.2.4 `CurrencyCode` Implemented as Heap String Instead of Closed Enum
- **Location:** [`crates/haven-domain/src/offer/currency.rs:11-13`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/offer/currency.rs#L11-L13)
  ```rust
  pub const SUPPORTED_CURRENCIES: &[&str] = &["NGN", "USD", "EUR", "GBP", "CAD", "AUD", "KES", "GHS"];
  pub struct CurrencyCode(String);
  ```
- **Violation:** A fixed set of 8 supported currencies is stored in a heap-allocated `String` with runtime string comparison against an array, preventing exhaustive pattern matching and wasting allocations.
- **Concrete Rust Fix:** Convert `CurrencyCode` into a 1-byte copyable enum:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub enum CurrencyCode {
      Ngn,
      Usd,
      Eur,
      Gbp,
      Cad,
      Aud,
      Kes,
      Ghs,
  }
  ```
  Implement `std::str::FromStr`, `Display`, and `as_str(&self) -> &'static str`.

---

### 1.3 Principle 6: Making Errors Useful
*Principle 6:* "Library crates: one `thiserror` enum per concern, a variant per distinct cause, with `#[source]` and identifying context... No unwrap()/expect()/panic! in non-test code except for a proven invariant."

#### 1.3.1 Monolithic Grab-Bag `DomainError`
- **Location:** [`crates/haven-domain/src/error.rs:4-46`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/error.rs#L4-L46)
- **Violation:** A single 47-line `DomainError` mixes account errors (`InvalidEmail`), offer validation errors (`BlankTitle`, `PriceRequired`, `InvalidPrice`), session errors (`TokenGenerationFailed`), and cursor errors (`InvalidCursor`).
- **Concrete Rust Fix:** Split into domain-focused error types:
  - `haven_domain::account::AccountError` (`InvalidEmail(String)`)
  - `haven_domain::offer::OfferValidationError` (`BlankTitle`, `TitleTooShort`, `PriceRequired`, etc.)
  - `haven_domain::session::SessionError` (`TokenGenerationFailed`)
  - `haven_domain::offer::cursor::CursorError` (`InvalidCursor`)

#### 1.3.2 `RepoError` Missing Identifying Context
- **Location:** [`crates/haven-domain/src/ports.rs:20-29`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-domain/src/ports.rs#L20-L29)
  ```rust
  pub enum RepoError {
      #[error("Resource not found or unauthorized")]
      NotFound,
      #[error("Conflict with existing resource")]
      Conflict,
      ...
  }
  ```
- **Violation:** `NotFound` and `Conflict` provide zero context regarding which entity, table, or ID was missing or collided.
- **Concrete Rust Fix:** Add structured context:
  ```rust
  pub enum RepoError {
      #[error("{entity} not found: {id}")]
      NotFound { entity: &'static str, id: String },
      #[error("{entity} conflict on key: {key}")]
      Conflict { entity: &'static str, key: String },
      ...
  }
  ```

#### 1.3.3 Panic via `assert!` in Library/Production Code
- **Location:** [`crates/haven-web/src/rate_limit.rs:40-43`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/rate_limit.rs#L40-L43)
  ```rust
  assert!(
      capacity > 0 && refill_per_sec > 0.0,
      "limiter needs a positive burst and refill rate"
  );
  ```
- **Violation:** Production constructor causes an unhandled panic on invalid input instead of returning a `Result`.
- **Concrete Rust Fix:** Return `Result<Self, RateLimitConfigError>` or use non-zero types (`NonZeroU32` for capacity) so invalid configurations fail at compile time.

---

### 1.4 Principle 8: Modularity & Visibility Ladder
*Principle 8:* "Default to private. Widen only as far as needed: `pub(super)` when only the parent module needs it, `pub(crate)` only for items intentionally shared across the crate, and `pub` only for the crate's deliberate public API. Internals should not be reachable from outside their module unless that reach is intended."

- **Locations:**
  - [`crates/haven-db/src/accounts.rs:7-9`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-db/src/accounts.rs#L7-L9): `pub struct PostgresAccountRepository { pub pool: DbPool }`
  - [`crates/haven-db/src/offers.rs:9-11`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-db/src/offers.rs#L9-L11): `pub struct PostgresOfferRepository { pub pool: DbPool }`
  - [`crates/haven-db/src/users.rs:7-9`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-db/src/users.rs#L7-L9): `pub struct PostgresUserRepository { pub pool: DbPool }`
  - [`crates/haven-db/src/magic_links.rs:7-9`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-db/src/magic_links.rs#L7-L9): `pub struct PostgresMagicLinkRepository { pub pool: DbPool }`
  - [`crates/haven-db/src/sessions.rs:7-9`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-db/src/sessions.rs#L7-L9): `pub struct PostgresSessionRepository { pub pool: DbPool }`
- **Violation:** Structs and their `pool` fields are declared `pub` at the crate boundary, even though external consumers only interact with `PostgresRegistry` via `haven_domain::ports` traits.
- **Concrete Rust Fix:**
  - Reduce repository structs to `pub(crate)`: `pub(crate) struct PostgresAccountRepository`.
  - Make inner pool fields private: `pool: DbPool`.

---

### 1.5 Principle 9: Hard to Break & Workspace Lints
*Principle 9:* "Workspace lints in `Cargo.toml` (`[workspace.lints]`): `unsafe_code = "forbid"`; clippy `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` at deny for non-test code; `wildcard_enum_match_arm` where decisions live."

- **Location:** [`Cargo.toml:20`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/Cargo.toml#L20)
  ```toml
  [workspace.lints.rust]
  unsafe_code = "deny"
  ```
- **Violation:** `unsafe_code` is configured as `"deny"` instead of `"forbid"`. Unlike `forbid`, `deny` permits local `#![allow(unsafe_code)]` overrides.
- **Concrete Rust Fix:** Change `unsafe_code = "deny"` to `unsafe_code = "forbid"`.

- **Location:** [`Cargo.toml:27-47`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/Cargo.toml#L27-L47)
- **Violation:** `clippy::wildcard_enum_match_arm` is missing from `[workspace.lints.clippy]`.
- **Concrete Rust Fix:** Add `wildcard_enum_match_arm = "deny"` to `[workspace.lints.clippy]`.

---

## 2. Haven UX Design Violations

### 2.1 Section 1 & 6: Destructive Deletion Without Confirmation
*UX Rule 1:* "No dead ends: every screen offers a way forward and a way back. After failure, say how to recover."  
*Contract ([`docs/DESIGN.md:130`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/docs/DESIGN.md#L130)):* `*Failure midway / Destructive:* Deletion requires explicit confirmation naming the offer to be removed before executing POST.`

- **Locations:**
  - [`crates/haven-web/src/app/offers/manage/id.rs:45-50`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/offers/manage/id.rs#L45-L50)
  - [`crates/haven-web/src/app/components/offer_card.rs:30-34`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/components/offer_card.rs#L30-L34)
  - [`crates/haven-web/src/app/offers/manage/id/delete.rs:3-18`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/offers/manage/id/delete.rs#L3-L18)
- **Violation:** Clicking "Delete" immediately dispatches a destructive `POST /offers/manage/{id}/delete`. There is no confirmation dialog, confirmation screen, or prompt naming the offer title. An accidental tap irrevocably deletes the offer.
- **Concrete UX Fix:**
  - Introduce `GET /offers/manage/{id}/delete` confirmation view displaying:
    - Heading: "Delete offer?"
    - Warning message explicitly naming the offer: "Are you sure you want to delete **{title}**? This cannot be undone."
    - Form with POST action and two actions: "Confirm Delete" (`ButtonVariant::Danger`) and "Cancel" link back to `/offers/manage/{id}`.
  - In `offer_card` and `manage/id`, link the delete button to `GET /offers/manage/{id}/delete` rather than directly posting.

---

### 2.2 Section 1 & 6: Form Validation Failures Drop to HTTP 400
*UX Rule 6:* "After a submission, the person sees confirmation or an error in the place they are looking, not elsewhere. Errors sit next to the field they concern."  
*Contract ([`docs/DESIGN.md:128`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/docs/DESIGN.md#L128)):* `*Error:* Validation errors re-render form with inputs preserved, error summary at top, and inline messages next to invalid fields.`

- **Locations:**
  - [`crates/haven-web/src/app/offers/manage/new.rs:86-100`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/offers/manage/new.rs#L86-L100):
    `create_req.validate().map_err(|e| topcoat::router::error::bad_request(e.to_string()))?;`
  - [`crates/haven-web/src/app/offers/manage/id/edit.rs:92-105`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/offers/manage/id/edit.rs#L92-L105):
    `domain_update.validate().map_err(|e| bad_request(e.to_string()))?;`
- **Violation:** When form validation fails (e.g. title too short, missing currency for positive price), the server returns an HTTP 400 Bad Request error page. All submitted user values are discarded, creating a dead end. Reusable components [`form_error`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/components/form_error.rs) and the `error` property on [`text_field`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/components/field.rs) are never rendered on POST failure.
- **Concrete UX Fix:** When validation fails in `create_offer` or `update_offer`, re-render the form view with HTTP 422 Unprocessable Entity:
  - Populate form fields with the submitted values (`value: Some(form.title)`).
  - Pass the validation error summary to `form_error(message: ...)`.
  - Pass field-specific error messages to `text_field(error: Some(...))`.

---

### 2.3 Section 1: Static Header Missing User State & Sign-Out Navigation
*UX Rule 1:* "One primary action per screen. The next step is always obvious. No dead ends: every screen offers a way forward and a way back."

- **Location:** [`crates/haven-web/src/app.rs:55-62`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app.rs#L55-L62) (`root_layout`)
  ```rust
  <header class="site-header">
      <nav class="nav-container">
          <a href="/" class="site-logo">"Haven"</a>
          <ul class="nav-links">
              <li><a href="/offers">"Offers"</a></li>
          </ul>
      </nav>
  </header>
  ```
- **Violation:**
  1. The header is completely static and unreactive to authentication state.
  2. Authenticated users have no header link to their dashboard (`/offers/manage`).
  3. Authenticated users have **no sign-out control anywhere in the application**. While `POST /auth/sign-out` exists in backend routes, there is no button or link to it, permanently trapping users in their session.
  4. Unauthenticated users browsing `/offers` have no header link to sign in.
- **Concrete UX Fix:**
  - Make `root_layout` inspect `crate::app::auth::guard::current_user(cx).await?`.
  - If authenticated: render navigation links for `"Offers"` (`/offers`), `"Your Offers"` (`/offers/manage`), and an inline form with a button or link for `"Sign Out"` (`POST /auth/sign-out`).
  - If unauthenticated: render `"Offers"` (`/offers`) and `"Sign In"` (`/auth/sign-in`).

---

### 2.4 Section 4 & 6: Missing `:active` (Pressed) Interaction States
*UX Rule 6:* "Every interactive element has hover, focus, active (pressed) and disabled states, styled consistently. Pressed state responds with no perceptible delay."

- **Location:** [`crates/haven-web/static/style.css:148-196`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/static/style.css#L148-L196)
- **Violation:** Zero `:active` rules exist in `style.css`. While `:hover`, `:focus-visible`, and `:disabled` are styled, buttons and links provide no immediate tactile visual depression feedback when pressed.
- **Concrete UX Fix:** Add `:active` rules for interactive buttons and links:
  ```css
  .btn:active {
    transform: translateY(1px);
  }
  .btn-primary:active {
    background-color: var(--color-accent-hover);
  }
  .btn-secondary:active {
    background-color: var(--color-border);
  }
  ```

---

### 2.5 Section 7 & DESIGN.md: Non-Greyscale Color Tokens
*Contract ([`docs/DESIGN.md:9-13`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/docs/DESIGN.md#L9-L13)):* "1. Greyscale only: black, white, and grays. No accent color... Without color: the primary action stands out by fill, weight, and position; errors and destructive actions carry words, icons, or placement; links are underlined; focus uses a thick high-contrast outline."

- **Location:** [`crates/haven-web/static/style.css:13-16, 68`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/static/style.css#L13-L16)
  ```css
  --color-danger: #dc2626;         /* Red */
  --color-danger-surface: #fef2f2; /* Light red */
  --color-danger-border: #fca5a5;  /* Red border */
  --color-focus: #2563eb;          /* Blue */
  ```
- **Violation:** Stylesheet introduces red (`#dc2626`) and blue (`#2563eb`) accent colors, violating Haven's confirmed monochrome-first design decision. (Flagged in `DESIGN.md:150` under Open Questions).
- **Concrete UX Fix:** Re-align role tokens to pure greyscale:
  ```css
  --color-focus: #111827;
  --color-danger: #111827;
  --color-danger-surface: #f3f4f6;
  --color-danger-border: #111827;
  ```
  Ensure focus outlines use `outline: 2px solid var(--color-focus); outline-offset: 2px;` to maintain WCAG 3:1 focus contrast without color.

---

### 2.6 Section 6 & Accessibility: Inputs Lack ARIA Error Association
*UX Rule 6 & WCAG AA:* Form input error states must be programmatically associated with form controls for assistive technology.

- **Location:** [`crates/haven-web/src/app/components/field.rs:27-45`](file:///home/tgenericx/dev/github.com/phastboy/haven/expts/rust/crates/haven-web/src/app/components/field.rs#L27-L45)
- **Violation:** When `error` is present, `<span class="field-error">` is rendered below the `<input>`, but the `<input>` lacks `aria-invalid="true"` and `aria-describedby`, so screen readers do not associate the error with the input field.
- **Concrete UX Fix:**
  - Add `aria-invalid=(error.is_some().then_some("true"))` to `<input>` and `<textarea>`.
  - Add `id=(format!("{name}-error"))` to `<span class="field-error">`.
  - Add `aria-describedby=(error.is_some().then_some(format!("{name}-error")))` to the input control.

---

## 3. Prioritized Remediation Roadmap

| Priority | Issue | Skill / Principle | Action Summary |
|---|---|---|---|
| **P0** | Trapping navigation & missing Sign-Out | UX Rule 1 | Make `root_layout` dynamic: add "Your Offers" & POST "Sign Out" for auth users; add "Sign In" for guests. |
| **P0** | Immediate destructive delete | UX Rule 1, 6 / DESIGN.md | Add `GET /offers/manage/{id}/delete` confirmation view naming offer title before deletion POST. |
| **P0** | Form errors drop to HTTP 400 | UX Rule 6 / DESIGN.md | Re-render form with HTTP 422, preserving inputs and displaying `form_error` and inline field errors. |
| **P1** | Hidden system clock in domain | Principle 3, 5 | Replace `Session::is_valid()` and `MagicLink::is_valid()` with pure `is_valid_at(now)`. |
| **P1** | Serde deserialization bypass | Principle 4 | Add `#[serde(try_from = "...")]` on `Price`, `OfferSlug`, `Email`, `CurrencyCode`. |
| **P1** | Public fields & Default on IDs | Principle 4 | Make ID fields private; remove impure random `Default` implementations on entity IDs. |
| **P2** | Monolithic `DomainError` | Principle 6 | Split `DomainError` into `AccountError`, `OfferValidationError`, `SessionError`, `CursorError`. |
| **P2** | Lints in `Cargo.toml` | Principle 9 | Set `unsafe_code = "forbid"` and add `wildcard_enum_match_arm = "deny"`. |
| **P2** | Overly broad DB visibility | Principle 8 | Downgrade repository structs to `pub(crate)` and make `pool` fields private. |
| **P3** | Non-greyscale tokens | UX Section 7 / DESIGN.md | Convert `--color-danger` and `--color-focus` to pure greyscale tokens. |
| **P3** | Missing `:active` states & ARIA | UX Rule 4, 6 | Add `.btn:active` CSS and `aria-invalid` / `aria-describedby` in `field.rs`. |
| **P3** | `CurrencyCode` heap string | Principle 4 | Refactor `CurrencyCode` into closed unit enum. |

---

## 4. Execution Plan: Milestones & Checklists

### Milestone 1: Workspace Safety & Compiler Guardrails
- [x] Upgrade `unsafe_code = "deny"` to `unsafe_code = "forbid"` in `[workspace.lints.rust]`.
- [x] Add `wildcard_enum_match_arm = "deny"` in `[workspace.lints.clippy]`.
- [x] Verify `cargo clippy --workspace --all-targets` passes without warnings.

### Milestone 2: Domain Layer Hardening (`haven-domain`)
- [x] **Clock Injection:**
  - [x] Refactor `Session::is_valid` to `Session::is_valid_at(&self, now: DateTime<Utc>) -> bool`.
  - [x] Refactor `MagicLink::is_valid` to `MagicLink::is_valid_at(&self, now: DateTime<Utc>) -> bool`.
- [x] **Encapsulation & Type Safety:**
  - [x] Make inner fields private for `OfferId`, `UserId`, `AccountId`, `SessionId`, and `IdempotencyKey`.
  - [x] Introduce `MagicLinkId(Uuid)`.
  - [x] Remove `impl Default` for entity IDs (`OfferId`, `UserId`, `AccountId`, `SessionId`).
- [x] **Serde Protection:**
  - [x] Add `#[serde(try_from = "...")]` to `Price`, `OfferSlug`, `Email`, and `CurrencyCode`.
- [x] **Currency Code Enum:**
  - [x] Convert `CurrencyCode` to a closed unit enum (`Ngn`, `Usd`, `Eur`, `Gbp`, `Cad`, `Aud`, `Kes`, `Ghs`).
- [x] **Error Architecture:**
  - [x] Split monolithic `DomainError` into per-concern error types (`AccountError`, `OfferValidationError`, `SessionError`, `CursorError`).
  - [x] Add structured entity and key context to `RepoError::NotFound` and `RepoError::Conflict`.
- [x] Run `cargo test -p haven-domain`.

### Milestone 3: Persistence Boundaries & Visibility (`haven-db`)
- [x] Enforce visibility ladder: reduce `PostgresAccountRepository`, `PostgresOfferRepository`, `PostgresUserRepository`, `PostgresMagicLinkRepository`, `PostgresSessionRepository` to `pub(crate)`.
- [x] Make `pool: DbPool` fields private across repository structs.
- [x] Adapt SQL query mappings to domain changes (private IDs, `CurrencyCode` enum, decomposed errors).
- [x] Run `cargo test -p haven-db`.

### Milestone 4: Critical User Flows & Interactive Feedback (`haven-web`)
- [x] **Navigation & Session State:**
  - [x] Update `root_layout` to inspect auth state: render "Offers", "Your Offers", and POST "Sign Out" for signed-in users; "Offers" and "Sign In" for guests.
- [x] **Destructive Deletion Confirmation:**
  - [x] Create `GET /offers/manage/{id}/delete` confirmation view naming the offer title.
  - [x] Wire "Delete" buttons in `offer_card` and `manage/id` to the confirmation view.
- [x] **Form Error Handling & Input Preservation:**
  - [x] Update `create_offer` and `update_offer` to re-render form on validation failure with HTTP 422.
  - [x] Populate `form_error` and inline field `error` props while preserving submitted inputs.
- [x] **Rate Limiter:**
  - [x] Refactor `RateLimiter::new` to eliminate panic-inducing `assert!`. Provide `RateLimiter::try_new` returning `Result<Self, RateLimiterError>`.

### Milestone 5: Visual Styling, Accessibility & Design Tokens
- [x] Convert `--color-danger`, `--color-danger-surface`, `--color-danger-border`, and `--color-focus` in `static/style.css` to pure greyscale tokens.
- [x] Add `.btn:active` and link active states in `static/style.css` for immediate tactile feedback.
- [x] Add `aria-invalid` and `aria-describedby` error associations in `crates/haven-web/src/app/components/field.rs`.

### Milestone 6: Full Verification & Audit Closeout
- [x] Run `cargo test --workspace` (all 88 tests passing: 29 in haven-domain, 1 in haven-db, 58 in haven-web).
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings` (clean, 0 warnings).
- [x] Run `cargo fmt --check` (clean formatting).
- [x] Run `cargo doc --workspace --no-deps` (clean documentation builds).
- [x] Update `CODE_DESIGN_AND_UX_AUDIT.md` to reflect 100% resolution.

---

## 5. Audit Closeout Summary

All 12 reported violations across Rust Code Design Principles and Haven UX Design have been 100% remediated on branch `refactor/code-design-and-ux-remediation`:

1. **Compiler & Safety Guardrails**: `unsafe_code = "forbid"` and `wildcard_enum_match_arm = "deny"` enforced at workspace level in `Cargo.toml`.
2. **Domain Isolation & Integrity**: Injected time (`is_valid_at(now)`), encapsulated IDs with private fields and explicit `from_uuid`/`as_uuid()` constructors, Serde `#[serde(try_from = "...")]` input bypass prevention, 1-byte unit enum `CurrencyCode`, and granular domain errors (`AccountError`, `OfferValidationError`, `SessionError`, `CursorError`, `RepoError::{NotFound, Conflict}`).
3. **Repository Boundaries**: Submodule repositories encapsulated as `pub(crate)` with private `pool` fields and crate-level constructor methods `new(pool)`.
4. **Interactive UX & Safe Flows**:
   - Dynamic navigation bar in `root_layout` reflecting auth state with prominent "Sign Out" POST action.
   - Dedicated `GET /offers/manage/{id}/delete` confirmation step protecting destructive offer removals.
   - Non-destructive form validation in `manage/new` and `manage/id/edit` returning HTTP 422 with input preservation and inline error banners.
   - Non-panicking rate limiter constructor with `try_new` fallible initialization.
5. **Monochrome Design System & Accessibility**:
   - Pure greyscale tokens (`--color-danger`, `--color-danger-surface`, `--color-danger-border`, `--color-focus`) per `docs/DESIGN.md`.
   - Tactile `:active` button pressed states.
   - Complete accessible input association via `aria-invalid="true"`, `aria-describedby`, and linked element IDs.
6. **Full Test & Lint Verification**:
   - `cargo test --workspace`: 88/88 passed.
   - `cargo clippy --workspace --all-targets -- -D warnings`: passed cleanly.
   - `cargo fmt --check`: passed cleanly.
   - `cargo doc --workspace --no-deps`: passed cleanly.

