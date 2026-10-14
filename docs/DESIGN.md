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

### 9. Route separation: public slug vs owner UUID (decided, 2026-10-08; refined 2026-10-12)
- **What:** public offer views live at `/offers/{slug}`; owner management lives at `/profile/offers/{id}`. Visiting by slug never questions ownership (no edit or delete controls displayed).
- **Why:** slugs are SEO-friendly and public. `/profile/offers/{id}` enforces authenticated ownership for editing and deletion. Probing other IDs yields identical 404 responses (IDOR prevention).
- **Replaces:** single overloaded route attempting to serve both public viewing and owner editing, and previously separate `/offers/manage/{id}` hierarchy.

### 10. Automated query-count test (decided, 2026-10-08)
- **What:** automated test verifying that rendering a list of offers runs in $O(1)$ database queries.
- **Why:** guards against future N+1 query regressions as domain relations are introduced.
- **Replaces:** unmonitored query growth across listing endpoints.

### 11. Transparent disabled state for unconfigured auth (decided, 2026-10-09)
- **What:** email magic link inputs and buttons are disabled with explicit affordance directing users to Google OAuth when mail infrastructure is unconfigured (`EMAIL_DELIVERY_ENABLED`).
- **Why:** allowing users to submit an email magic link without active delivery causes them to wait indefinitely for emails that never arrive, eroding trust.
- **Replaces:** silently accepting email input when email delivery is not operational.

### 12. Minimal header navigation (decided, 2026-10-10)
- **What:** top-level header navigation contains only primary high-level navigation destinations: "Offers" and "Sign In" (guests) or "Offers" and "Profile" (authenticated users). Inline forms and sub-actions ("Sign Out", "Your Offers") are moved into the profile dashboard.
- **Why:** avoids cluttering the top bar with competing actions and cognitive noise. Respects UX hierarchy where session termination and owner management live under the account context.
- **Replaces:** header carrying "Offers", "Your Offers", "Profile", and an inline "Sign Out" button side by side.

### 13. Profile as shell with modular dashboard sections (decided, 2026-10-10; refined 2026-10-12)
- **What:** `/profile` serves as a persistent shell with tabbed navigation: Overview (`/profile`), Offers (`/profile/offers`), and Account (`/profile/account`). Profile editing is isolated at `/profile/edit`. Offer creation and management live under `/profile/offers/*`.
- **Why:** avoids monolithic profile screens by treating sections as modular children within an identity shell. Improving the offers section touches only its own component, preserving shell and account stability.
- **Replaces:** pouring inventory, forms, identity, and sign-out into a single unstructured dashboard.

### 14. Maker identity attribution slot (decided, 2026-10-10)
- **What:** offer cards in the public feed and the public offer detail page display a clean maker attribution slot ("by <maker_name>") linking directly to `/profiles/{handle}`.
- **Why:** Haven is not an anonymous price-competing listing site; it is a place where people are known by what they offer. Responders can evaluate maker identity and view everything else they offer.
- **Replaces:** offers without visible author attribution.

### 15. Zero-JavaScript Popover confirmation dialog (decided, 2026-10-12)
- **What:** offer deletion confirmation on `/profile/offers/{id}` uses the native HTML Popover API with `<dialog id="delete-offer-dialog" popover="auto" class="confirm-dialog">`, triggered by `<button popovertarget="delete-offer-dialog">`, cancelled by `<button popovertarget="delete-offer-dialog" popovertargetaction="hide">`, and submitted via `<form method="post" action="/profile/offers/{id}/delete">`.
- **Why:** eliminates a separate delete confirmation GET route while strictly upholding Haven's zero-JavaScript principle. The browser's native Popover API manages light-dismiss (clicking outside or pressing Escape) and keyboard focus without client-side scripts.
- **Replaces:** dedicated `/offers/manage/{id}/delete` confirmation GET screen.

### 16. Canonical OAuth callback and dead-route elimination (decided, 2026-10-12)
- **What:** `/auth/google/redirect` is the sole canonical Google OAuth callback URI matching production configuration. Pruned duplicate `/auth/google/callback`, bare collection redirect `/profiles`, and unused `/offers/new`.
- **Why:** visitors follow links and affordances rendered by the application rather than typing URLs. Removing duplicate routes eliminates maintenance surface and ambiguity.
- **Replaces:** duplicate route aliases and convenience redirects.

### 17. Consistent post-action destinations (decided, 2026-10-12)
- **What:** authenticated visitors navigating to `/`, `/auth/sign-in`, or completing OAuth/magic-link authentication always land on `/offers`. Post-action form redirects return directly to the originating section: offer create/edit/delete return to `/profile/offers`; profile edit returns to `/profile`.
- **Why:** predictable destinations reinforce the user's mental model and eliminate jarring transitions across disparate views.
- **Replaces:** arbitrary redirects sending creators to public detail views or landing pages.

### 18. Navigation layout rule: Main vs Child layout (decided, 2026-10-12)
- **What:** Exactly two layouts govern all navigation chrome across the application:
  1. **Main Layout (`NavigationLayout::Main`):** Used exclusively by top-level routes (`/offers`, `/profile`, and `/`). Shows the primary site navigation (`.site-header`) on all screen sizes. Never shows a back button header.
  2. **Child Layout (`NavigationLayout::Child`):** Used by every other route (e.g. `/offers/{slug}`, `/profiles/{handle}`, `/profile/edit`, `/profile/offers`, `/profile/offers/new`, `/profile/offers/{id}`, `/profile/offers/{id}/edit`, `/profile/account`, `/auth/*`).
     - **On mobile (`<= 640px`):** Primary navigation is completely hidden (`.site-header.desktop-only`). The header contains only a clean back button (`.child-header` with an explicit SVG back icon and "Back" label, meeting the 44px touch target).
     - **On large screens (`> 640px`):** Primary navigation stays visible because wide screens have ample room and hiding it saves no space. The back header stays visible underneath primary navigation, preserving consistent back-navigation affordance across devices.
  3. **Back Navigation Resolution ("Where you came from" default, Parent fallback):**
     - The default destination for the back button is always where the user came from, resolved from the request's same-site `Referer` header. The application sets `Referrer-Policy: same-origin` to ensure internal navigation paths are sent by browsers without leaking referrers to external origins.
     - If `Referer` is absent (direct link, bookmark, privacy blocker), or if it matches the current path (refresh loop guard), or is cross-origin, the back button falls back to the route's configured parent.
     - For public profiles (`/profiles/{handle}`), when viewed by the authenticated owner (`handle == current_user_handle`), the fallback parent is `/profile`. When viewed by others/guests, the fallback parent is `/offers`.
     - Protocol-relative (`//`), `javascript:`, and non-HTTP URLs are strictly rejected to eliminate open redirects.
  4. **Explicit SVG Icons Only (No Character Signs):** Navigation and slot affordances must never use unicode text characters/signs (e.g. `"←"`, `"›"`) pretending to be icons. Back navigation uses an explicit SVG arrow-left icon (`.back-icon-svg`) and row trailing slots use an explicit SVG chevron icon (`.row-chevron-svg`).
  5. **Modularity Contract:** A page never builds its own navigation header or ad-hoc back link. Its route is classified into a layout at the root shell (`crates/haven-web/src/app/navigation.rs`), ensuring adding or updating a route never touches existing view templates.
- **Why:** On small mobile screens, top-level navigation competes with focused task completion and wastes critical vertical space; on wide screens, persistent primary navigation prevents disorientation. Explicit SVG icons scale predictably without font rendering variance or alignment drift across platforms.
- **Replaces:** Ad-hoc text signs and back links placed inconsistently within individual page templates (`← Back to all offers`, `← Back to your offers`, `← Back to offer`) and showing redundant primary navigation on mobile child views.

### 19. Borderless editorial cards with SEO stretched links and fold-aware lazy loading (decided, 2026-10-14)
- **What:** Public offer listings use borderless, content-first cards separated by vertical whitespace and an inset hairline divider (`border-bottom: 1px solid var(--color-border)`). Cards feature a 4:3 hero media slot with `--radius-md` (8px) corners, maker identity row (avatar badge with computed initials e.g. "EV" and link to `/profiles/{handle}`), prominent headline title, and prominent price (e.g. "Free", "28 USD", "40 NGN" - strictly avoiding currency symbols). Real SEO-friendly linking wraps the title in an anchor stretched over the entire card hit area (`.offer-title-link::after`), leaving maker links and owner action buttons at `z-index: 2` independently clickable. Cover images above the fold (items < 2) load eagerly with `fetchpriority="high"`, while items below the fold use `loading="lazy"` with explicit dimensions (`width="320" height="240"`, `aspect-ratio: 4 / 3`) to eliminate layout shifts and conserve Cloudinary transformation budget.
- **Why:** Replaces boxy bordered cards with Airbnb-style editorial restraint where content is the decoration. Real semantic anchors ensure SEO crawlers and screen readers find public offer slugs, while CSS stretched-link preserves tap-friendly mobile ergonomics without client-side JavaScript.
- **Replaces:** Boxed cards (`border: 1px solid var(--color-border)`), uniform lazy loading, and text-only maker attribution.

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

### 3. Owner Offers Section & Command Center (`/profile/offers`, `/profile/offers/{id}`)
- **Stage & Lane:** Stage 0.1.x / 0.3.x, Offerer lane (managing own inventory).
- **Purpose:** Offerer manages their existing offers (list, create, edit, delete) within the `/profile` shell.
- **Content used:** Owner's offers list and forms (`title`, `price`, `currency`, `description`, `id`).
- **Hierarchy:** On offers list: Primary action is "Create an Offer" (`/profile/offers/new`). Per item: "Manage" button links to `/profile/offers/{id}`. On command center (`/profile/offers/{id}`): "Edit Offer" and "View Public" are secondary actions, "Delete Offer" opens the Popover confirmation dialog. On forms: Primary action is submission button.
- **State matrix:**
  - *Empty:* "No offers yet. Create your first offer to make it available to others." with prompt and action to create one.
  - *Typical:* Keyset-paginated list of owner's own offers; command center displays summary with Edit, View Public, and Delete buttons.
  - *Long text:* Descriptions clamped on list; full textarea on edit.
  - *Error:* Validation errors re-render form with inputs preserved, error summary at top, and inline messages next to invalid fields.
  - *Forbidden or not found:* Unauthenticated visits redirect to `/auth/sign-in`. Requests for offers owned by others return 404 (IDOR prevention).
  - *Destructive / Popover:* Deletion triggers native `<dialog popover="auto">` with confirmation prompt, Cancel button, and POST form. Light-dismiss via Escape or backdrop click without JavaScript.

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

### 5. Public Maker Profile (`/profiles/{handle}`)
- **Stage & Lane:** Stage 0.3.x, Responder lane (evaluating maker identity and catalog).
- **Purpose:** Anyone (guest, crawler, or signed-in user) can view a maker's identity, bio, and all available offers they have created.
- **Content used:** `User.display_name`, `User.handle`, `User.bio`, and list of `Offer` items created by the user.
- **Hierarchy:** Primary emphasis is the maker's identity (`display_name`, `@handle`, bio). Secondary section is their offers feed (`Offers by <display_name>`). Quiet elements: back link to all offers.
- **State matrix:**
  - *Typical:* Maker header with display name, @handle, and bio (if provided), followed by a list of their available offers.
  - *Empty (no offers):* "No offers yet" ("This person has not posted any offers yet. Check back later.") calm empty state message.
  - *No bio:* Bio paragraph collapses neatly without leaving awkward empty gaps.
  - *Long text:* Display name wraps cleanly; long bio renders in comfortable reading measure (45–75 chars).
  - *Not found (404):* Returns 404 with clean not-found message when handle does not exist.
  - *Forbidden:* N/A (public route; no auth required).

### 6. Owner Profile Shell (`/profile`), Profile Edit (`/profile/edit`), & Account (`/profile/account`)
- **Stage & Lane:** Stage 0.3.x, Offerer lane (identity & account management).
- **Purpose:** Authenticated user views their identity overview within the profile shell, jumps to their public profile, edits profile details, or terminates their session from the Account section.
- **Content used:** `User.display_name`, `User.handle`, `User.bio`, `User.email`.
- **Hierarchy:** Profile shell provides persistent tabs: Overview, Offers, Account. Within Overview (`/profile`): Primary action is "Edit Profile" (`/profile/edit`); secondary is "View Public Profile" (`/profiles/{handle}`). The Edit form (`/profile/edit`) features a "Save Profile" primary submit button alongside a "Cancel" link back to `/profile`. Finally, Account (`/profile/account`) provides a primary "Sign Out" form button (POST `/auth/sign-out`).
- **State matrix:**
  - *Typical Overview:* Identity card with display name, handle, bio, "Edit Profile" CTA, and public profile link.
  - *Typical Account:* Identity summary with email and session termination card ("Sign Out").
  - *Edit form typical:* Form pre-filled with current display name, handle, and bio.
  - *Edit validation error:* Errors displayed inline above each field (e.g. handle taken, invalid chars, display name too short) and in error banner.
  - *Updated banner:* Success notice rendered when arriving back via `/profile?updated=1`.
  - *Unauthenticated:* Automatically redirects to `/auth/sign-in`.

## Open questions

- Gray values for the light theme, and the contrast check for each text and background pair (WCAG AA: ≥ 4.5:1 for body text, ≥ 3:1 for large text and UI boundaries/focus outlines).
- Whether any status or accent need appears as stages are built (the rule in decision 1 applies: color only when a named job cannot be done by weight, position, or spacing).
- Typeface revisit when brand identity work begins (decision 3: self-hosted font swap at `--font-sans`).
- Component stylesheet refactoring: aligning existing `style.css` rules (e.g. `--color-danger`, `--color-focus`) to pure greyscale tokens.
