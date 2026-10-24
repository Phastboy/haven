# Offers Feed Redesign Specification & Implementation Blueprint

## Source of Truth & Visual Reference
- **Visual Reference**: `docs/design/offers-feed.html` (Stitch export saved on branch `feat/offers-feed-responsive`).
- **Principles to extract**: Layout structure, responsive grid proportions, spacing scales, tokens, interactive states, and plain language copy.
- **Principles to discard from mock**:
  - Google Fonts (`Inter`) & Google Material Symbols (blocked by CSP `default-src 'self'`).
  - Inline styles & client-side `onclick` JavaScript.
  - Presentation frames, viewport headers, and hardcoded placeholder images.

---

## Architectural & Craftsmanship Constraints

1. **Rendering & Scripts**:
   - 100% Server-Side Rendered (Topcoat SSR), zero client-side JavaScript.
   - Icons are crisp inline SVGs (`viewBox="0 0 24 24"`, `stroke="currentColor"`, `stroke-width="2"`, `fill="none"` or `fill="currentColor"`).
   - Fonts use the existing system stack (`var(--font-sans)`).
   - Colors and spacing leverage existing `:root` greyscale design tokens in `static/style.css`.

2. **Grid & Overflow Defense**:
   - Desktop (1440px): `repeat(4, minmax(0, 1fr))` with `gap: 24px`.
   - Tablet (1024px): `repeat(3, minmax(0, 1fr))` with `gap: 20px`.
   - Mobile (390px): `repeat(2, minmax(0, 1fr))` with `gap: 12px`.
   - Cards strictly declare `min-width: 0;` so single-line `white-space: nowrap; text-overflow: ellipsis;` never widens a grid column.

3. **Function & Module Standards (`rust-craft`)**:
   - Maximum 3 parameters per function; group component props into typed structs.
   - Component files isolated under `crates/haven-web/src/app/components/`.
   - Existing `offer_card` remains intact for owner management and public profiles.
   - Narrative commits: one atomic, passing commit per step.
   - Each step concludes with:
     ```bash
     cargo fmt --all
     cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
     cargo test --workspace --locked
     cargo sqlx prepare --workspace (if SQL was modified)
     ```

---

## Phased Implementation Sequence

### Step 1: Title Limit Reduction
- **Domain**: In `crates/haven-domain/src/offer/validation.rs`, reduce `MAX_TITLE_CHARS` from `100` to `50`.
- **Domain Error**: Update `OfferValidationError::TitleTooLong` display message to `"title must not exceed 50 characters"`.
- **Web Views**: Update `maxlength` from `100` to `50` in `crates/haven-web/src/app/profile/offers/new.rs` and `crates/haven-web/src/app/profile/offers/id/edit.rs`.
- **Database**: No schema constraint added (historical offers >50 chars remain valid and truncate via card CSS).
- **Tests**: Update character boundary tests in `crates/haven-domain/src/offer/tests.rs` from 100/101 to 50/51 chars.

### Step 2: Square Card Photo Transformations
- **Image Module**: In `crates/haven-web/src/app/components/image.rs`, add square transformation helpers:
  - `t_card_sq` (1x: 320x320) and `t_card_sq_2x` (2x: 640x640).
  - Add `card_sq_url(cloud_name, public_id)` and `card_sq_srcset(cloud_name, public_id)`.
- **Backwards Compatibility**: Retain existing `t_card` / `t_card_2x` helpers unchanged for legacy `offer_card`.

### Step 3: Batch Saved Offers Lookup (Eliminating N+1 Queries)
- **Domain Port**: In `crates/haven-domain/src/ports.rs`, extend `SavedOfferRepository`:
  ```rust
  async fn saved_among(&self, user_id: UserId, offer_ids: &[OfferId]) -> Result<HashSet<OfferId>, RepoError>;
  ```
- **Database Adapter**: In `crates/haven-db/src/saved.rs`, implement `saved_among` using a single parameterized query:
  ```sql
  SELECT offer_id FROM saved_offer WHERE user_id = $1 AND offer_id = ANY($2)
  ```
- **Fake Adapter**: Implement `saved_among` in `crates/haven-domain/src/fakes.rs`.
- **Usage**: The `/offers` feed calls `saved_among` once per page solely when a signed-in viewer exists, passing the IDs of the fetched page items. Never call `is_saved` in a loop per card.
- **Tests**: Add unit and integration tests verifying lookup accuracy and empty slice handling.

### Step 4: Safe Return Path for Save Action
- **Handler**: In `crates/haven-web/src/app/offers/slug/save.rs`, extend `SaveForm`:
  ```rust
  pub struct SaveForm {
      pub action: String,
      pub return_to: Option<String>,
  }
  ```
- **Validator**: Add a pure validation function:
  - Must be a same-site relative path starting with `"/offers"`.
  - Strictly reject protocol schemes (`http:`, `https:`), scheme-relative prefixes (`//`), backslashes (`\`), CRLF characters, or non-offers paths.
  - Return validated path or fall back to `/offers/{slug}`.
- **Tests**: Dedicated unit tests asserting accepted and rejected URL variations (e.g. `//evil.com`, `/profile`, `javascript:`, `/offers?q=test`).

### Step 5: New UI Components
- **`crates/haven-web/src/app/components/save_heart.rs`**:
  - Props struct: `slug: OfferSlug`, `is_saved: bool`, `return_to: Option<String>`, `is_authenticated: bool`.
  - For authenticated viewer: POST form to `/offers/{slug}/save` with hidden `action` (`"save"` or `"unsave"`), hidden `return_to`, and circular button (32x32) with inline SVG heart (outline when unsaved, filled when saved) and accessible `aria-label`.
  - For unauthenticated viewer: anchor link styled as the same circular button pointing to `/auth/sign-in`.
- **`crates/haven-web/src/app/components/listing_card.rs`**:
  - Props struct: `title`, `href`, `price_text`, `maker_name`, `maker_href`, `cover_url`, `cover_srcset`, `is_saved`, `return_to`, `is_authenticated`, `is_above_fold`.
  - Square 1:1 image container with rounded corners (12px).
  - Floating `save_heart` anchored to top-right corner with contrast drop-shadow.
  - Three fixed-height single-line text slots:
    1. Title: medium weight, truncated with ellipsis, wrapped with a stretched link overlay (`stretched-link`).
    2. Maker name: muted plain text, truncated with ellipsis.
    3. Price: formatted currency string or `"Free"`.
  - Heart sits on a higher stacking context (`z-index: 2`) so interacting with the heart does not trigger navigation.
- **Component Tests**: Render tests verifying markup, labels, escaping, and authenticated vs guest states.

### Step 6: Feed Integration & Style Finalization
- **Navigation Shell**: Responsive left rail on desktop (240px) / collapsed rail on tablet (72px) / bottom tab bar on mobile (56px).
- **Feed Assembly**: Update `crates/haven-web/src/app/offers/feed.rs` to render `listing_card`s in the responsive CSS grid.
- **Filters**: Direct filter bar entry without redundant `"Offers"` page heading; mobile price disclosure; thin 2px black border-left validation error banner.
- **Styles**: Complete the CSS token rules in `static/style.css`.
