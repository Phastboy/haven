# Haven Design Log

## Intent & Aesthetic Principles

- **Text-led restraint**: Content is the decoration. No decorative gradients, glowing borders, or arbitrary drop shadows. Inspired by Airbnb's restraint with clear typographic hierarchy.
- **Monochrome first**: High-contrast black, white, and neutral grays. Hierarchy is established through font size, font weight, positioning, and whitespace alone. Color is used strictly for accents (primary actions, focus rings) and meaningful states (danger, warning, success).
- **Semantic HTML & Strict CSP**: Semantic elements (`<header>`, `<main>`, `<nav>`, `<form>`, `<label>`, `<button>`, `<ul>`, `<li>`). No inline CSS (`style=""`) or `<style>` tags to strictly comply with Haven's Content Security Policy (`default-src 'self'`).
- **All states designed**: Every screen explicitly accounts for empty, typical, long text, validation error, server error, and not found states.

---

## Token System (`references/tokens.md`)

| Role | Token | Value / Definition | Reason |
|---|---|---|---|
| Page Background | `--color-bg` | `#ffffff` | Quiet, readable canvas |
| Raised Surface | `--color-surface` | `#f9fafb` | Subtle structural grouping |
| Primary Text | `--color-text` | `#111827` | High contrast (at least 4.5:1) |
| Secondary Text | `--color-text-muted` | `#6b7280` | Hierarchy without font size changes |
| Border / Divider | `--color-border` | `#e5e7eb` | Subtle division without heavy boxes |
| Accent | `--color-accent` | `#111827` (monochrome) / dark charcoal | Single accent for primary actions and focus |
| Accent Text | `--color-accent-text`| `#ffffff` | High contrast on accent buttons |
| Danger | `--color-danger` | `#dc2626` | Destructive confirmations (delete) |
| Focus Ring | `--color-focus` | `#2563eb` | Clear visible outline on interactive controls |

### Typography Scale (Major Third 1.25)
- `--text-sm`: `0.875rem` (14px) — metadata, captions, help text
- `--text-base`: `1rem` (16px) — body text, inputs
- `--text-lg`: `1.25rem` (20px) — card titles, subheadings
- `--text-xl`: `1.5625rem` (25px) — page headings
- Measure: 45–75 characters per line for reading and forms.
- System font stack: `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`.

---

## Component Inventory (Planned for v0.2.x)

1. **`button`**:
   - `primary`: solid background (`--color-accent`), white text (`--color-accent-text`).
   - `secondary`: bordered outline (`--color-border`), muted text.
   - `danger`: destructive actions, calm styling until confirmed.
2. **`field`**:
   - Visible `<label>`, formatted `<input>` or `<textarea>`, helper text, and inline error message.
3. **`form_error`**:
   - Summary block at top of forms when submission has validation errors.
4. **`offer_card` / `offer_row`**:
   - Slot-based layout: Title (heading), Price (badge/meta), Description (line-clamped), Link to public slug.
5. **`pagination_nav`**:
   - Keyset cursor controls (`Next`, `Previous`) without page drift.
6. **`empty_state`**:
   - Calm message explaining what belongs here and the single next action.

---

## Screen Inventory & State Matrices

### 1. Public Offer Feed (`/offers` or `/`)
- **Purpose**: A person can browse available offers without needing an account.
- **States**:
  - *Empty*: "No offers available yet." with a brief explanation and prompt to sign in or create an offer.
  - *Typical*: 20 items per page with title, formatted price, clamped description, and link to public view.
  - *Paginated*: Next / Previous buttons using keyset cursors.
  - *Long text*: Titles wrap cleanly, descriptions line-clamped to 2 lines.

### 2. Public Offer Detail (`/offers/{slug}`)
- **Purpose**: Anyone (guest, crawler, or owner) can see the full presentation of an offer by its slug.
- **States**:
  - *Typical*: Full title, price, currency, complete description, timestamp. No owner management buttons.
  - *Not found (404)*: Clean message ("Offer not found") with link back to available offers.

### 3. Owner Management (`/offers/manage`, `/offers/manage/{id}`)
- **Purpose**: Offerer manages their existing offers (edit, delete).
- **States**:
  - *List*: Owner's own offers with edit/delete links.
  - *Edit*: Pre-filled form with patch update semantics.
  - *Delete*: Dedicated confirmation screen naming the offer to be deleted.

### 4. Authentication Screens (`/auth/sign-in`, `/auth/sent`, `/auth/verify`)
- **Purpose**: Enter email or use Google OAuth to obtain a session.
- **Styling**: Centered calm card, visible labels, clear feedback.

---

## Decision Log

- **2026-10-08: Keyset pagination on `(created_at, id)`**:
  - *Why*: Offset pagination skips or duplicates rows as new offers are created concurrently. Keyset paging guarantees stable, constant-time indexed seeks.
- **2026-10-08: Public views use `slug`; `/offers/manage/{id}` reserved for owner**:
  - *Why*: Slugs are SEO-friendly and public. Visiting by slug never questions ownership, even for the owner. `/offers/manage/{id}` enforces authenticated ownership for editing and deletion.
- **2026-10-08: Automated query-count test**:
  - *Why*: Enforces that rendering a list of offers runs in $O(1)$ queries, guarding against future N+1 regressions as relations are introduced.
- **2026-10-08: Central stylesheet (`/style.css`) with strict CSP compliance**:
  - *Why*: Eliminates inline styles and `<style>` blocks to maintain Haven's strict CSP header.
