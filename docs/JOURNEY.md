# Haven journey

One story, two lanes. Lanes are roles, not people: the **Offerer** (the person making an offer) and the **Responder** (the person responding to it). One person can play both. Stages follow `ROADMAP.md`; only the current stage is detailed.

Status: draft. Only entries under **Decided** were confirmed by the owner. Everything else is open or an observation from the code.

## The story at a glance

| Stage | Question | Offerer lane | Responder lane | Handoff | Status | Load target |
|---|---|---|---|---|---|---|
| 0.1.x | Can I offer something? | create account, sign in, create / see own / edit / delete an offer | none yet | none | completed | met historical baseline (see PERFORMANCE_CONTRACT.md) |
| 0.2.x | Can someone see it? | offer becomes public on create (generates slug); manage own via `/offers/manage/{id}` | see available offers (public feed), see another person's offer (`/offers/{slug}`) | an offer becomes visible to others | completed | met ≥ 1,200 iters/s host direct |
| 0.3.x | Can I know who is behind an offer? | has profile, edits profile, profile displays made offers | sees who made an offer, visits public profile, sees maker's offers | knowing who is behind an offer | completed | met target; keyset pagination |
| 0.4.x | Can someone act on it? | know about expressed interest | express willingness to take on an offer | commitment | current | |
| 0.5.x | Can someone find it? | | find an offer by name, narrow by price | | not reached | |
| 0.6.x | Can I come back to what I care about? | | save an offer, follow a person, return to both | | not reached | |
| 0.7.x | Can I narrow it down? | | narrow by category, tags, saved, committed | | not reached | |
| 0.8.x to 0.9.x | TBD | | | | not reached | |
| 1.0.x | Can two people close an offer? | | | agreement, closing | not reached | |
| 1.x | TBD | | | | not reached | |
| 2.0.x | Can Haven handle a complete trade? | | | payment, fulfillment, completion | not reached | |

```mermaid
sequenceDiagram
  participant O as Offerer
  participant H as Haven
  participant R as Responder
  O->>H: create account, sign in
  O->>H: create an offer (generates slug)
  O->>H: edit profile (display name, bio)
  R->>H: see available offers (public feed)
  R->>H: see another person's offer (/offers/{slug}) with maker link
  Note over R,H: 0.3.x handoff: knowing who is behind an offer
  R->>H: visit maker's public profile (/profiles/{id_or_handle})
  H-->>R: show maker profile & all their offers
```

## Stage 0.3.x: Can I know who is behind an offer? (completed)

### Untangled items

**A person can have a profile that represents them.**
- Every platform participant (User) has a profile representation.
- Profile fields: display name (required for public identity, e.g. 2–50 chars), bio/about (optional, e.g. up to 500 chars), and public profile handle or slug.
- Profile state on first sign-in: either initialized with a default derived from email prefix, or an empty profile state prompting the person to set their display name.
- Identity is text-led, restrained, and clean (no noisy gamification or arbitrary badges per `haven-ui-design`).

**A person can edit their profile.**
- An authenticated person navigates to their profile management view (`/profile/edit`).
- Only the owner can edit their own profile (IDOR protection; CSRF protected).
- Can update display name, bio, and handle/slug. Input validation enforces character bounds and sanitization.

**A person can see who made an offer.**
- On `/offers/{slug}` (and offer feed cards), the offer displays the maker's display name and link to their public profile.
- Unauthenticated guests and signed-in responders can see the maker without needing special permissions.
- If the maker has not set a custom bio, their display name is still clearly presented.

**A person can visit another person's public profile.**
- Anyone can request the public profile via a clean URL (`/profiles/{handle}`).
- Zero authentication required: public, cacheable, and SEO-friendly.
- Displays the person's display name, bio/about, and account creation/member-since metadata.

**A person's profile shows the offers they have made.**
- On the public profile page, a dedicated section lists all available offers created by that person.
- Uses keyset cursor pagination on `(created_at, id)` scoped to `user_id` to prevent pagination drift.
- Each offer card displays title, price/free, currency, and links to `/offers/{slug}`.
- If the person has no available offers, an honest empty state is shown ("No offers available right now").

### Decided
- 2026-10-10: Profile representation includes `display_name` (2–50 chars), optional `bio` (up to 500 chars), and a unique URL-safe `handle` (3–30 lowercase alphanumeric + hyphens).
- 2026-10-10: Public profiles are routed at `/profiles/{handle}` (e.g. `/profiles/alex`).
- 2026-10-10: On first sign-in, `display_name` and `handle` are automatically initialized from the email prefix (falling back with a unique suffix if taken), and can be edited anytime by the owner.
- 2026-10-10: Maker attribution with a link to their public profile appears on both the single offer view (`/offers/{slug}`) and each card in the main public feed (`/offers`).
- 2026-10-10: Separation of profile view and edit: `/profile` is the owner dashboard (identity overview, direct link to public profile, "Your Offers" management, and sign-out form); `/profile/edit` is the dedicated edit form.
- 2026-10-10: Header decluttering: Top-level navigation is kept minimal ("Offers" and "Sign In" or "Profile"). "Sign Out" and "Your Offers" live within the `/profile` dashboard.

### Open questions
- **Blocking:**
  - None. All key product decisions for 0.3.x are confirmed.
- **Non-blocking:**
  - Initials avatar styling vs pure text display name.
  - Reserved handles list (e.g., `manage`, `new`, `edit`, `admin`, `api`, `auth`, `offers`, `profiles`).

### Deferred dependencies
- Following a person. Belongs to 0.6.x. Current behaviour: no follow button or follower count.
- Direct communication with the maker. Belongs to 0.4.x. Current behaviour: view maker's offers only.

### Done when (from the roadmap)
A person looking at an offer can find out who made it, and from there see everything else that person offers, and the system meets its load-testing target.

## Stage 0.2.x: Can someone see it? (completed)

### Untangled items

**A person can see available offers.**
- Anyone (unauthenticated guest, crawler, or authenticated user) visits the public offer feed without needing to sign in.
- Shows available offers in reverse chronological order (`created_at DESC, id DESC`).
- Uses keyset cursor pagination on `(created_at, id)` with a capped page size (default 20, max 50). Keyset paging prevents skipped or repeated rows as new offers are created.
- Queries execute in $O(1)$ query count (verified by automated query-count tests).
- Each row or card displays title, price (or "Free"), currency (if priced), shortened description, and links to its public view via `slug`.
- Visual presentation follows `haven-ui-design` (monochrome first, semantic design tokens, clean typographic hierarchy, Airbnb-inspired restraint).
- States:
  - Empty: "No offers available yet" with a calm explanation and a link to create an offer (if authenticated or guiding to sign in).
  - Typical: list of offer cards.
  - Many items / Paginated: clear "Next" (and "Previous") cursor controls that maintain position without page drift.
  - Long text: titles wrap cleanly, descriptions are clamped.
  - Loading / Slow response: server-rendered fast response; no layout shift.

**A person can see another person's offer.**
- Anyone requests `/offers/{slug}` (or the public slug path).
- Zero authentication required: public and SEO-friendly.
- Does NOT question ownership: even if visited by the creator, the public view is rendered without edit/delete controls.
- Displays full title, price, currency, complete description, and creation metadata.
- Owner controls (edit, delete) reside exclusively at `/offers/manage/{id}` where ownership is verified.
- States:
  - Typical: clean, text-led presentation of the offer.
  - Not found: unknown slug yields an informative 404 page with a link back to available offers.
  - Long content: long descriptions (up to 2,000 characters) render with comfortable line length (45–75 characters).
  - Offer deleted: if deleted while being viewed, subsequent interactions or reloads cleanly yield 404.

## Stage 0.1.x: Can I offer something?  (completed)

### Untangled items

**A person can create an account and sign in.** Checked in the roadmap. Completed via Magic Link and Google OAuth with adaptive session cookies.

**A person can create an offer.**
- Offerer is signed in and opens the create form.
- Fields: title (3 to 100 characters), description (10 to 2000 characters), price in minor units (0 or more, required). Currency from a fixed list (NGN, USD, EUR, GBP, CAD, AUD, KES, GHS) is required only when price is greater than zero; for zero-price offers, it may be omitted or blank and defaults to NGN.
- Enforced at domain and DB constraints.

**A person can see their own offers.**
- Offerer sees a list of their own offers. In 0.2.x, this view is hardened with keyset pagination and styled using semantic tokens.

**A person can edit an offer.**
- Offerer opens `/offers/manage/{id}/edit` and modifies any subset of fields. Patch semantics preserve unchanged values.

**A person can delete an offer.**
- Offerer deletes their own offer after confirmation.

### Decided

- 2026-10-08: Public view of an offer is accessed via unique URL `slug` (e.g. `/offers/{slug}`) without authentication or ownership checks (SEO-friendly).
- 2026-10-08: `/offers/manage/{id}` (by UUID) is reserved for authenticated owner management and enforces ownership.
- 2026-10-08: Public and owner offer listings use keyset cursor pagination on `(created_at, id)` with a capped page size to eliminate skip/repeat anomalies.
- 2026-10-08: A query-count test enforces that listing offers executes in $O(1)$ queries to prevent N+1 regressions.
- 2026-10-08: Load-testing gate for 0.2.x is ≥ 1,200 iterations/sec executed directly on the host machine.
- 2026-10-08: All pages and components (including previously unstyled forms and views) will be styled following `haven-ui-design`: monochrome first, semantic design tokens, semantic HTML, and strict CSP adherence.
- 2026-10-08: A price of 0 means free, for now (owner: provisional wording, may change later). Displayed as "Free".
- 2026-10-08: On create, an empty price field is a validation error; a person must explicitly enter 0 for a free offer.
- 2026-10-08: In 0.1.x a person can list all of their own offers and create, edit, and delete them.
- 2026-10-08: In 0.1.x a person cannot list another person's offers or request a particular one. Requesting another person's offer behaves like requesting one that does not exist.
- 2026-10-08: On update, old values are preserved; only the person's edits are written.
- 2026-10-09: Route separation: public offers live at `/offers/{slug}` with shared caching (ETag), canonical URL tags, no session read, and no maker profile details.
- 2026-10-09: Owner management routes are prefixed under `/offers/manage` (`/offers/manage`, `/offers/manage/new`, `/offers/manage/{id}`, `/offers/manage/{id}/edit`, delete).
- 2026-10-09: Owner routes enforce auth via shared helper `require_owner_auth`, setting `Cache-Control: private, no-store` and `noindex`. `robots.txt` disallows `/offers/manage/`.
- 2026-10-09: Probing IDs is prevented by returning identical 404 responses for nonexistent offers and offers owned by others.
- 2026-10-09: Slugs always end in their id suffix and reserve words `manage` and `new`.

### Open questions

**Blocking** (the stage's "Done when" or the next screen depends on these):
- None. Routing, ownership isolation, and slug collision structures are resolved.

**Non-blocking:**
- Default page size: default 20 items, max 50 items per page?
- Keyset cursor encoding: base64-encoded `(created_at, id)` or compound string format?
- Header navigation: how should public header differentiate between signed-in and guest states?

### Deferred dependencies

- What happens to an offer that someone has already committed to when its owner edits or deletes it. Belongs to 0.3.x. Current behaviour until then: not defined; no commitments exist in 0.2.x.
- Search by name, filtering by price or category. Belongs to 0.4.x and 0.6.x. Current behaviour: reverse chronological public feed only.

### Done when (from the roadmap)

An offer created by one person can be seen by another person, and the system meets its load-testing target.
