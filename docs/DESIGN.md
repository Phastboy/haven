# Haven design decisions

Applies to the Rust web app (Topcoat server-rendered views). Method: `haven-ui-design` skill. Every entry records what was decided, why, and what it replaces. Add entries just in time, when a screen first needs them.

Status key: **decided** (owner confirmed), **default** (assistant's choice because the owner delegated it; revisit when a reason appears).

## Global decisions

### 1. Greyscale only (decided, 2026-10-10)
- **What:** black, white, and grays. No accent color.
- **Why:** structure carries the meaning (size, weight, position, spacing). The owner saw greyscale output and liked it, and two reference designs reinforced it.
- **Rule for change:** a color is added only when a named job cannot be done by weight, position, or spacing. Record the job here when adding one.
- **Without color:** the primary action stands out by fill, weight, and position; errors and destructive actions carry words, icons, or placement; links are underlined; focus uses a thick high-contrast outline.

### 2. Light theme first (decided, 2026-10-10)
- **What:** ship the light theme. Dark comes later by redefining the same role tokens under `prefers-color-scheme`, with no component changes.
- **Why:** public, text-led pages read by strangers on any device; Airbnb is the same (light first).
- **Constraint:** components use role tokens only (`--color-bg`, `--color-surface`, `--color-text`, `--color-text-muted`, `--color-border`, `--color-focus`), never raw values, so the second theme stays cheap.

### 3. System font stack (default, 2026-10-10)
- **What:** `system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`. Monospace, if ever needed: `ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`.
- **Why:** no font download, so pages load faster and nothing shifts while loading; no external request, so the strict CSP stays simple; no licensing; reads natively on each device.
- **Trade-off:** Haven looks slightly different per device, and a custom typeface is part of a brand's identity (Airbnb uses one).
- **Revisit when:** identity and brand become work, which the roadmap's profile stage points toward. The font is one token (`--font-sans`), so the swap touches one place. If a typeface is chosen, self-host it (no third-party font service).

### 4. Text-led UI (decided, 2026-10-10)
- **What:** hierarchy comes from size, weight, color role (primary vs muted), and spacing, because offers have no images.
- **Replaces:** photo-led layouts (Airbnb, bakery list) for now.

### 5. One row component with fixed slots (decided, 2026-10-10)
- **What:** a repeated item (an offer in a list) is one component with named, optional slots in fixed positions: leading, title, description (muted, clamped to two lines), value, status, action. An absent slot collapses and the others keep their alignment.
- **Why:** consistent alignment makes a list read as one object, and the same structure can hold other content later.
- **Rule:** add a slot only when the schema has data for it. Today an offer has title, description, price with currency, so there is no leading or status slot yet. A price of 0 shows as "Free".

### 6. Modular structure (decided, 2026-10-10)
- **What:** components take data in and return markup; pages assemble them; layers are tokens, base, components, pages.
- **Why:** improving one component must not mean editing the page that assembles it.

## Architectural & route decisions

### 7. Central stylesheet with strict CSP (decided, 2026-10-08)
- **What:** one stylesheet served from `/style.css`. Semantic HTML only (`header`, `main`, `nav`, `form`, `label`, `button`, `ul`). Zero inline styles (`style=""`) or `<style>` blocks.
- **Why:** eliminates inline styles and `<style>` blocks to strictly comply with Haven's Content Security Policy (`default-src 'self'`).
- **Replaces:** inline styles and per-template CSS blocks.

### 8. Keyset pagination on `(created_at, id)` (decided, 2026-10-08)
- **What:** keyset cursor pagination (`after`) on `(created_at, id)` with capped page size (default 20, max 50).
- **Why:** offset pagination skips or duplicates rows as new offers are created concurrently. Keyset paging guarantees stable, constant-time indexed seeks.
- **Replaces:** offset-based (`page=N`) pagination.

### 9. Route separation: public slug vs owner UUID (decided, 2026-10-08)
- **What:** public offer views live at `/offers/{slug}`; management lives at `/offers/manage/{id}`. Visiting by slug never questions ownership (no edit or delete controls displayed).
- **Why:** slugs are SEO-friendly and public. `/offers/manage/{id}` enforces authenticated ownership for editing and deletion. Probing other IDs yields identical 404 responses (IDOR prevention).
- **Replaces:** single overloaded route attempting to serve both public viewing and owner editing.

### 10. Automated query-count test (decided, 2026-10-08)
- **What:** automated test verifying that rendering a list of offers runs in $O(1)$ database queries.
- **Why:** guards against future N+1 query regressions as domain relations are introduced.
- **Replaces:** unmonitored query growth across listing endpoints.

### 11. Transparent disabled state for unconfigured auth (decided, 2026-10-09)
- **What:** email magic link inputs and buttons are disabled with explicit affordance directing users to Google OAuth when mail infrastructure is unconfigured (`EMAIL_DELIVERY_ENABLED`).
- **Why:** allowing users to submit an email magic link without active delivery causes them to wait indefinitely for emails that never arrive, eroding trust.
- **Replaces:** silently accepting email input when email delivery is not operational.

## References: borrowed and rejected

| Reference | Borrowed | Rejected | Why |
|---|---|---|---|
| Airbnb | restraint, content as decoration, consistent cards, progressive disclosure | photo-led layout, custom typeface (for now) | Haven has no images yet; a custom typeface is an open brand question |
| Bakery list (Pinterest) | fixed-slot row, muted description, value as strongest secondary element | photo, stock pill, circular action button, shadows, large radii | no images or status exist; pill and shadow are decoration here |
| Light/dark phone mockup (Pinterest) | colors by role, inverted high-contrast fill for the primary action | decorative dashboard tiles, tiny text, low-contrast muted text on dark | role tokens make a second theme cheap; the rejected parts fail contrast or add nothing |

## Tokens

Starting proposals live in the skill's `references/tokens.md`. A token becomes decided only when logged here with its reason.

### Decided tokens
- **Role tokens (Decision 2):** `--color-bg`, `--color-surface`, `--color-text`, `--color-text-muted`, `--color-border`, `--color-focus`.
- **Typography font stack (Decision 3):** `--font-sans` (`system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`). Monospace fallback: `ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`.

### Not yet decided
- Gray hex values for the light theme and contrast verification for each text/background pair (see Open questions).
- Type sizes, line heights, and typography scale ratio.
- Spacing scale (`--space-1` through `--space-7`).
- Radii (`--radius-sm`, `--radius-md`, `--radius-full`).
- Max content width and line measure.

## Screens

For each screen, add: purpose (stage and lane), content used (from the schema), hierarchy (one primary action), and a state matrix (empty, typical, long text, error, forbidden or not found, failure midway).

### 1. Public Offer Feed (`/offers` or `/`)
- **Stage & Lane:** Stage 0.2.x, Responder lane (first exposure to available offers).
- **Purpose:** A person can browse available offers without needing an account.
- **Content used:** `Offer.title`, `Offer.price`, `Offer.currency` (or "Free"), `Offer.description` (clamped), `Offer.slug` (from `haven_domain::offer::Offer`).
- **Hierarchy:** Primary action is selecting an offer card to view its public detail (`/offers/{slug}`). Secondary action is keyset pagination ("Next"). Quiet elements: header branding.
- **State matrix:**
  - *Empty:* "No offers available yet." with a calm explanation and prompt to sign in or create an offer.
  - *Typical:* 20 items per page with title, formatted price badge, description clamped to 2 lines, linking to `/offers/{slug}`.
  - *Long text:* Titles wrap cleanly; descriptions clamp to two lines without overflowing.
  - *Error:* Calm notification block explaining offers could not be loaded; suggests refreshing.
  - *Forbidden or not found:* N/A (public feed; zero authentication required).
  - *Failure midway:* If keyset cursor is invalid or pagination seek fails, falls back gracefully to the first page.

### 2. Public Offer Detail (`/offers/{slug}`)
- **Stage & Lane:** Stage 0.2.x, Responder lane (evaluating an offer).
- **Purpose:** Anyone (guest, crawler, or owner) can see the full presentation of an offer by its slug without authentication.
- **Content used:** Full `Offer.title`, `Offer.price`, `Offer.currency`, complete `Offer.description` (up to 2,000 characters), `Offer.created_at`.
- **Hierarchy:** Primary action is reading the offer content. Secondary action is back navigation to available offers. No owner management actions (edit/delete) are displayed on public view.
- **State matrix:**
  - *Typical:* Full title, price badge, full text description, creation timestamp.
  - *Long text:* Full description rendered comfortably within 45–75 character measure.
  - *Not found (404):* Clean "Offer not found" message with a link back to available offers.
  - *Error:* Server error renders a calm error page without internal IDs or stack traces.
  - *Forbidden:* N/A (public route; never checks ownership).
  - *Failure midway:* If an offer is deleted while being viewed, subsequent interactions or reloads cleanly render 404.

### 3. Owner Management (`/offers/manage`, `/offers/manage/{id}`)
- **Stage & Lane:** Stage 0.1.x / 0.2.x, Offerer lane (managing own inventory).
- **Purpose:** Offerer manages their existing offers (list, create, edit, delete).
- **Content used:** Owner's offers list and forms (`title`, `price`, `currency`, `description`, `id`).
- **Hierarchy:** On list: Primary action is "Create an Offer" (`/offers/manage/new`). Per item: "Edit" is secondary, "Delete" is calm secondary button. On forms: Primary action is submission button.
- **State matrix:**
  - *Empty:* "You haven't created any offers yet." with prompt and action to create one.
  - *Typical:* List of owner's own offers with edit and delete links; edit form pre-filled with existing data.
  - *Long text:* Descriptions clamped on list; full textarea on edit.
  - *Error:* Validation errors re-render form with inputs preserved, error summary at top, and inline messages next to invalid fields.
  - *Forbidden or not found:* Unauthenticated visits redirect to `/auth/sign-in`. Requests for offers owned by others return 404 (IDOR prevention).
  - *Failure midway / Destructive:* Deletion requires explicit confirmation naming the offer to be removed before executing POST.

### 4. Authentication Screens (`/auth/sign-in`, `/auth/sent`, `/auth/verify`)
- **Stage & Lane:** Stage 0.1.x, Offerer & Responder lanes (session acquisition).
- **Purpose:** Enter email or use Google OAuth to obtain an authenticated session.
- **Content used:** Google OAuth CTA, email input (disabled when delivery is unconfigured), status messages.
- **Hierarchy:** Primary action is "Continue with Google" (while email delivery is unconfigured). Form fields and informational notices remain quiet.
- **State matrix:**
  - *Typical:* Centered card with visible labels and Google OAuth button.
  - *Disabled / Unconfigured:* Email input and submit disabled with explicit notice directing users to Google OAuth.
  - *Long text:* Email inputs and provider labels fit cleanly within card bounds.
  - *Error:* Verification failure (expired/invalid token) shows clear explanation and single button to request a new link.
  - *Forbidden or not found:* Clean redirect to sign-in.
  - *Failure midway:* OAuth callback failure returns to sign-in with clear error notice.

## Open questions

- Gray values for the light theme, and the contrast check for each text and background pair (WCAG AA: ≥ 4.5:1 for body text, ≥ 3:1 for large text and UI boundaries/focus outlines).
- Whether any status or accent need appears as stages are built (the rule in decision 1 applies: color only when a named job cannot be done by weight, position, or spacing).
- Typeface revisit when brand identity work begins (decision 3: self-hosted font swap at `--font-sans`).
- Component stylesheet refactoring: aligning existing `style.css` rules (e.g. `--color-danger`, `--color-focus`) to pure greyscale tokens.
