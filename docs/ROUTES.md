# Haven Route Catalog and Affordance Guide

This document catalogs every route currently available in Haven, its HTTP method, access tier, functional purpose, UI affordances, and failure states.

---

## Navigation Layout Rule

Haven enforces a strict two-layout rule across all routes:

1. **Top-Level Routes (`NavigationLayout::Main`):** `/offers`, `/profile`, and `/`.
   - **Mobile (`<= 640px`):** Main layout renders primary navigation (`.site-header`). No back button is shown.
   - **Desktop (`> 640px`):** Main layout renders primary navigation.
2. **Child Routes (`NavigationLayout::Child`):** Every other route (e.g. `/offers/{slug}`, `/profiles/{handle}`, `/profile/edit`, `/profile/offers`, `/profile/offers/new`, `/profile/offers/{id}`, `/profile/offers/{id}/edit`, `/profile/account`, `/auth/*`).
   - **Mobile (`<= 640px`):** Primary navigation is completely hidden (`.site-header.desktop-only`). The header contains only a clean back button (`.child-header` with an explicit SVG arrow-left icon and "Back" label).
   - **Desktop (`> 640px`):** Primary navigation remains visible (wide screens have room for it and hiding it saves no space). The back header remains visible below the primary navigation for consistent back-navigation affordance across devices.
3. **Safe Back URL Resolution:**
   - The default destination is always the page the user came from using the validated same-site `Referer` request header (`Referrer-Policy: same-origin` is enforced).
   - If `Referer` is absent, cross-site, protocol-relative (`//`), or matches the current URL (browser refresh loop guard), it falls back safely to the child's configured parent route (`/offers`, `/profile`, etc.).
   - Individual page templates never construct their own header or ad-hoc back link. Route classification occurs centrally in `crates/haven-web/src/app/navigation.rs`.

---

## Route Overview at a Glance

| Route Pattern | Method | Access Tier | Layout | Fallback Parent | Purpose | Primary Affordance |
|---|---|---|---|---|---|---|
| `/` | `GET` | Public | Main | — | Landing page | "Browse Offers" / "Sign In" (redirects authed to `/offers`) |
| `/offers` | `GET` | Public | Main | — | Available offers feed | Keyset pagination, offer cards, maker links |
| `/offers/{slug}` | `GET` | Public | Child | `/offers` | Single offer detail | View full offer description, maker link |
| `/profiles/{handle}` | `GET` | Public | Child | `/profile` (owner) / `/offers` | Public maker profile | Maker bio, catalog of maker's offers |
| `/auth/sign-in` | `GET`, `POST` | Public / Guest | Child | `/` | Sign-in portal | Email magic link, Google OAuth CTA (redirects authed to `/offers`) |
| `/auth/sent` | `GET` | Public | Child | `/` | Confirmation notice | Notice to check email inbox |
| `/auth/verify` | `GET`, `POST` | Public (token) | Child | `/` | Magic link verification | "Confirm and Sign In" token exchange (redirects to `/offers`) |
| `/auth/google` | `GET` | Public | Child | `/` | Google OAuth entrypoint | Generates CSRF state, redirects to Google |
| `/auth/google/redirect` | `GET` | Public | Child | `/` | Google OAuth callback | Canonical callback, exchanges code, redirects to `/offers` |
| `/auth/sign-out` | `POST` | Authenticated | Child | `/` | Terminate session | Clears session cookie, redirects to sign-in |
| `/profile` | `GET` | Authenticated | Main | — | Profile overview (shell) | Identity summary, "Edit Profile", "View Public Profile" |
| `/profile/edit` | `GET`, `POST` | Authenticated | Child | `/profile` | Edit owner profile | Display name, handle, bio form |
| `/profile/offers` | `GET` | Authenticated | Child | `/profile` | Owner offers catalog | "Create an Offer", keyset pagination, manage links |
| `/profile/offers/new` | `GET`, `POST` | Authenticated | Child | `/profile/offers` | Create an offer | Title, description, price, currency form |
| `/profile/offers/{id}` | `GET` | Authenticated | Child | `/profile/offers` | Owner offer command center | View own offer, edit, view public, zero-JS Popover delete |
| `/profile/offers/{id}/edit` | `GET`, `POST` | Authenticated | Child | `/profile/offers` | Edit own offer | Patch form for title, description, price, currency |
| `/profile/offers/{id}/delete` | `POST` | Authenticated | Child | `/profile/offers` | Delete own offer | Deletion submission (triggered from Popover dialog) |
| `/profile/account` | `GET` | Authenticated | Child | `/profile` | Account & session | Account details, "Sign Out" form button |
| `/style.css` | `GET` | Public | Static | — | Application stylesheet | CSS design tokens & semantic styles |
| `/health` | `GET` | Public | System | — | Uptime / Health check | HTTP 200 health verification |
| `/robots.txt` | `GET` | Public | System | — | Crawl instructions | Disallows indexing of `/profile/` |

---

## Detailed Route Specifications

### 1. Discovery & Public Pages

#### `GET /` — Landing Page
- **Access Tier:** Public (all visitors).
- **Behavior:**
  - If visitor is authenticated: automatically redirects (HTTP 303 See Other) to `/offers`.
  - If visitor is unauthenticated (guest): displays Haven introduction ("Exchange offers directly").
- **Affordances:**
  - Primary button: "Browse Offers" (`/offers`).
  - Secondary button: "Sign In" (`/auth/sign-in`).

#### `GET /offers` — Public Offers Feed
- **Access Tier:** Public.
- **Behavior:**
  - Lists published offers in reverse-chronological order using stable keyset cursor pagination on `(created_at, id)`.
  - Executes in $O(1)$ database query count joining maker identity.
- **Affordances:**
  - **Offer Card Title:** Clickable headline linking to `/offers/{slug}`.
  - **Maker Attribution Slot:** Text "by" followed by `<a href="/profiles/{handle}">Maker Name</a>`.
  - **Price / Value Badge:** Formatted price and currency (e.g. `50 USD`) or "Free" (for 0-price offers).
  - **Pagination Controls:** "Next →" and "← Previous" links preserving cursor position without drift.
  - **Empty State:** Calm message ("No offers available yet") when zero offers are listed.

#### `GET /offers/{slug}` — Public Offer Detail
- **Access Tier:** Public.
- **Behavior:**
  - Resolves offer by unique URL slug.
  - Zero authentication check; public and SEO-friendly with ETag caching support.
  - Preserves strict separation between public view and owner management: displays NO edit/delete buttons even if requested by the creator.
- **Affordances:**
  - **Headline & Price:** Full title and price badge.
  - **Maker Attribution:** "Offered by <a href="/profiles/{handle}">Maker Name</a>".
  - **Description Body:** Complete description rendered in comfortable 45–75 character measure.
  - **Back Navigation:** Provided by child layout header (`← Back` returning to previous page or falling back to `/offers`).
- **Failure States:**
  - Non-existent slug returns clean HTTP 404 Not Found page with link back to available offers.


#### `GET /profiles/{handle}` — Public Maker Profile
- **Access Tier:** Public.
- **Behavior:**
  - Looks up maker by `@handle`.
  - Fetches maker's public identity and list of available offers created by them in $O(1)$ query count.
- **Affordances:**
  - **Back Navigation:** Provided by child layout header (`← Back` returning to previous page or falling back to `/offers`).
  - **Maker Identity:** Maker's display name (`h1`), handle badge (`@handle`), and bio paragraph (omitted neatly if empty).
  - **Maker's Catalog:** Dedicated section ("Offers by {display_name}") listing all active offers created by this maker.
  - **Catalog Offer Cards:** Title links to `/offers/{slug}`, price, description, keyset pagination.
  - **Empty State:** "No offers yet" ("This person has not posted any offers yet. Check back later.") if maker has no published offers.
- **Failure States:**
  - Non-existent handle returns HTTP 404 Not Found.

---

### 2. Authentication & Session

#### `GET /auth/sign-in` — Sign-In Portal
- **Access Tier:** Public (guests; authenticated users redirect to `/offers`).
- **Affordances:**
  - **Email Magic Link Form:** Email input field and "Send Magic Link" primary button. When email delivery is not operational (`EMAIL_DELIVERY_ENABLED=false`), form fields and button are disabled with an explicit notice directing users to Google OAuth.
  - **Google OAuth CTA:** "Continue with Google" button linking to `/auth/google`.

#### `POST /auth/sign-in` — Dispatch Magic Link
- **Access Tier:** Public.
- **Behavior:**
  - Validates email address.
  - Generates secure random token, hashes it, stores in database with 15-minute expiration.
  - Dispatches email (if mailer configured) and redirects (HTTP 303) to `/auth/sent`.

#### `GET /auth/sent` — Magic Link Sent Notice
- **Access Tier:** Public.
- **Affordances:**
  - Informational message confirming link was sent.
  - "Back to Sign In" link.

#### `GET /auth/verify` — Magic Link Confirmation Prompt
- **Access Tier:** Public (requires `?token=...` query parameter).
- **Behavior:**
  - Renders verification landing card with `Referrer-Policy: no-referrer`.
  - Protects against email security pre-fetchers and scanners consuming the one-time token prematurely.
- **Affordances:**
  - "Confirm and Sign In" button submitting POST form with hidden token value.

#### `POST /auth/verify` — Magic Link Token Exchange
- **Access Tier:** Public (submits token form).
- **Behavior:**
  - Hashes submitted token and verifies against `magic_links` table.
  - Checks expiration and consumed status.
  - Creates authenticated session, writes HTTP-only `sid` session cookie, marks link consumed, and redirects to `/offers`.
- **Failure States:**
  - Expired, consumed, or invalid token returns verification error view with "Request a new link" button.

#### `GET /auth/google` — Google OAuth Initiation
- **Access Tier:** Public.
- **Behavior:**
  - Generates random CSRF state token and sets in HTTP-only short-lived cookie (`oauth_state`).
  - Issues HTTP 303 redirect to Google OAuth 2.0 authorization consent endpoint.
  - If Google OAuth is unconfigured, falls back to `/auth/sign-in`. If already signed in, redirects to `/offers`.

#### `GET /auth/google/redirect` — Google OAuth Callback (Canonical)
- **Access Tier:** Public.
- **Behavior:**
  - Matches registered production OAuth redirect URI. (Deprecated `/auth/google/callback` was pruned to eliminate duplicate endpoints).
  - Reads `code` and `state` from query parameters.
  - Verifies state against `oauth_state` cookie (CSRF prevention).
  - Exchanges authorization code for access token via Google token endpoint.
  - Fetches user info from Google UserInfo endpoint (email, display name).
  - If user exists by email, authenticates; if user is new, provisionally assigns default handle and display name.
  - Creates authenticated session, sets `sid` cookie, and redirects to `/offers`.

#### `POST /auth/sign-out` — Session Termination
- **Access Tier:** Authenticated.
- **Behavior:**
  - Invalidates current session record from database.
  - Clears `sid` cookie.
  - Issues HTTP 303 redirect to `/auth/sign-in`.
  - Accessible via the Account section of the `/profile/account` view.

---

### 3. Owner Profile Shell & Dashboard Sections

Haven wraps all owner capabilities inside a unified `/profile` shell. A persistent header displays the owner's identity (display name and handle) alongside section navigation tabs:
- **Overview:** `/profile`
- **Offers:** `/profile/offers`
- **Account:** `/profile/account`

Active tabs receive visual emphasis and `aria-current="page"`. Child sections render inside this shell while maintaining isolated responsibilities.

#### `GET /profile` — Profile Overview
- **Access Tier:** Authenticated (unauthenticated requests redirect to `/auth/sign-in`).
- **Affordances:**
  - **Identity Card:** Displays display name, handle (`@handle`), bio, and link to public profile (`/profiles/{handle}`).
  - **Edit Identity Affordance:** "Edit Profile" link button leading to `/profile/edit`.
  - **Update Notice:** Confirmation banner ("Your profile has been updated.") when redirected from edit with `?updated=1`.

#### `GET /profile/edit` — Owner Profile Edit Form
- **Access Tier:** Authenticated.
- **Affordances:**
  - Pre-filled text input: `display_name` (2 to 50 characters).
  - Pre-filled text input: `handle` (3 to 30 lowercase alphanumeric characters and hyphens).
  - Pre-filled textarea: `bio` (optional, up to 500 characters).
  - "Save Profile" primary submit button (POST `/profile/edit`).
  - "Cancel" link returning to `/profile`.
  - Inline error alerts for field-level validation errors (e.g. handle already taken, invalid characters).

#### `POST /profile/edit` — Process Profile Update
- **Access Tier:** Authenticated.
- **Behavior:**
  - Validates `display_name`, `handle`, and `bio` against domain value objects.
  - Executes atomic update in database with unique handle collision handling.
  - On success: redirects (HTTP 303) to `/profile?updated=1`.
  - On validation or collision error: re-renders form with existing inputs preserved and inline error banners.

#### `GET /profile/offers` — Owner Offers Catalog
- **Access Tier:** Authenticated.
- **Affordances:**
  - Section header: "Your Offers" with description.
  - "Create an Offer" primary button linking to `/profile/offers/new`.
  - Keyset-paginated list of owner's offers with title, formatted price, date, and "Manage" button linking to `/profile/offers/{id}`.
  - Empty state with clear messaging and "Create your first offer" CTA when zero offers exist.

#### `GET /profile/offers/new` — Create Offer Form
- **Access Tier:** Authenticated.
- **Affordances:**
  - Text input for `title` (3 to 100 characters).
  - Textarea for `description` (10 to 2,000 characters).
  - Number input for `price` (minor units; 0 for free).
  - Select dropdown for `currency` (NGN, USD, EUR, GBP, CAD, AUD, KES, GHS; required if price > 0).
  - "Create Offer" submit button.
  - "Cancel" link back to `/profile/offers`.
- **Behavior:**
  - On validation error: re-renders form with values preserved and error messages.
  - On success: creates offer and redirects (HTTP 303) to `/profile/offers`.

#### `POST /profile/offers/new` — Process Offer Creation
- **Access Tier:** Authenticated.
- **Behavior:**
  - Validates inputs, creates offer in database associated with current owner.
  - Redirects (HTTP 303) to `/profile/offers`.

#### `GET /profile/offers/{id}` — Owner Offer Command Center
- **Access Tier:** Authenticated (owner only; probing non-existent IDs or IDs owned by others returns HTTP 404).
- **Affordances:**
  - Summary display of offer: title, price, description, date, and link to public view (`/offers/{slug}`).
  - "Edit Offer" secondary button linking to `/profile/offers/{id}/edit`.
  - "View Public" link pointing to `/offers/{slug}`.
  - "Back to Your Offers" link pointing to `/profile/offers`.
  - **Zero-JavaScript Popover Delete Affordance:**
    - "Delete Offer" destructive button with `popovertarget="delete-offer-dialog"`.
    - Native modal dialog: `<dialog id="delete-offer-dialog" popover="auto" class="confirm-dialog">`.
    - Dialog contents: clear warning prompt, "Cancel" button with `popovertarget="delete-offer-dialog" popovertargetaction="hide"`, and `<form method="post" action="/profile/offers/{id}/delete"><button type="submit" class="btn-danger">Yes, Delete Offer</button></form>`.
    - Dismisses automatically on Escape key or backdrop click via native HTML Popover API with zero JavaScript.

#### `GET /profile/offers/{id}/edit` — Edit Offer Form
- **Access Tier:** Authenticated (owner only).
- **Affordances:**
  - Pre-filled inputs for title, description, price, and currency.
  - "Save Changes" submit button.
  - "Cancel" link back to `/profile/offers/{id}`.

#### `POST /profile/offers/{id}/edit` — Process Offer Update
- **Access Tier:** Authenticated (owner only).
- **Behavior:**
  - Applies partial patch updates while preserving untouched fields.
  - Validates domain rules and updates database.
  - Redirects (HTTP 303) to `/profile/offers`.

#### `POST /profile/offers/{id}/delete` — Process Offer Deletion
- **Access Tier:** Authenticated (owner only).
- **Behavior:**
  - Enforces ownership and removes offer record from database.
  - Redirects (HTTP 303) to `/profile/offers`.

#### `GET /profile/account` — Account & Session Overview
- **Access Tier:** Authenticated.
- **Affordances:**
  - Signed-in identity details (display name and handle).
  - Session termination card with "Sign Out" form button (POST `/auth/sign-out`).

---

### 4. System & Infrastructure

#### `GET /health` — Health Check
- **Access Tier:** Public.
- **Response:** HTTP 200 OK text body `ok`.
- **Purpose:** Used by Docker/Kubernetes container probes and cloud health checks.

#### `GET /style.css` — Central Stylesheet
- **Access Tier:** Public.
- **Response:** HTTP 200 OK stylesheet.
- **Purpose:** Serves CSS tokens and component styles with strict CSP (`default-src 'self'`). Zero external network dependencies.

#### `GET /robots.txt` — Search Engine Directives
- **Access Tier:** Public.
- **Response:**
  ```text
  User-agent: *
  Disallow: /profile/
  ```
- **Purpose:** Allows web spiders to index public content while disallowing private management areas.
