# Haven Route Catalog and Affordance Guide

This document catalogs every route currently available in Haven, its HTTP method, access tier, functional purpose, UI affordances, and failure states.

---

## Route Overview at a Glance

| Route Pattern | Method | Access Tier | Purpose | Primary Affordance |
|---|---|---|---|---|
| `/` | `GET` | Public | Landing page | "Browse Offers" / "Sign In" |
| `/offers` | `GET` | Public | Available offers feed | Keyset pagination, offer cards, maker links |
| `/offers/{slug}` | `GET` | Public | Single offer detail | View full offer description, maker link |
| `/offers/new` | `GET`, `POST` | Public | Convenience redirect | Redirects to `/offers/manage/new` |
| `/profiles` | `GET` | Public | Defensive redirect | Redirects to `/offers` |
| `/profiles/{handle}` | `GET` | Public | Public maker profile | Maker bio, catalog of maker's offers |
| `/auth/sign-in` | `GET`, `POST` | Public / Guest | Sign-in portal | Email magic link, Google OAuth CTA |
| `/auth/sent` | `GET` | Public | Confirmation notice | Notice to check email inbox |
| `/auth/verify` | `GET`, `POST` | Public (token) | Magic link verification | "Confirm and Sign In" token exchange |
| `/auth/google` | `GET` | Public | Google OAuth entrypoint | Generates CSRF state, redirects to Google |
| `/auth/google/callback` | `GET` | Public | Google OAuth callback | Exchanges OAuth code, creates session |
| `/auth/google/redirect` | `GET` | Public | Google OAuth alias | Alias for `/auth/google/callback` |
| `/auth/sign-out` | `POST` | Authenticated | Terminate session | Clears session cookie, redirects to sign-in |
| `/profile` | `GET` | Authenticated | Owner profile dashboard | Profile preview, "Edit Profile", "Your Offers", "Sign Out" |
| `/profile/edit` | `GET`, `POST` | Authenticated | Edit owner profile | Display name, handle, bio form |
| `/offers/manage` | `GET` | Authenticated | Owner offers catalog | "Create an Offer", edit/delete offer links |
| `/offers/manage/new` | `GET`, `POST` | Authenticated | Create an offer | Title, description, price, currency form |
| `/offers/manage/{id}` | `GET` | Authenticated | Owner offer detail | View own offer, edit/delete actions |
| `/offers/manage/{id}/edit` | `GET`, `POST` | Authenticated | Edit own offer | Patch form for title, description, price, currency |
| `/offers/manage/{id}/delete` | `GET`, `POST` | Authenticated | Delete own offer | Confirmation screen & delete submission |
| `/style.css` | `GET` | Public | Application stylesheet | CSS design tokens & semantic styles |
| `/health` | `GET` | Public | Uptime / Health check | HTTP 200 health verification |
| `/robots.txt` | `GET` | Public | Crawl instructions | Disallows indexing of `/offers/manage/` |

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
  - **Back Navigation:** "← Back to all offers" link (`/offers`).
- **Failure States:**
  - Non-existent slug returns clean HTTP 404 Not Found page with link back to available offers.

#### `GET /offers/new` & `POST /offers/new` — Redirect Routes
- **Access Tier:** Public.
- **Behavior:**
  - `GET /offers/new` issues HTTP 303 See Other redirecting to `/offers/manage/new`.
  - `POST /offers/new` issues HTTP 301 Moved Permanently redirecting to `/offers/manage/new`.

#### `GET /profiles` — Defensive Redirect
- **Access Tier:** Public.
- **Behavior:**
  - Redirects (HTTP 303 See Other) to `/offers` to prevent dead-end browsing when visitors request the parent directory.

#### `GET /profiles/{handle}` — Public Maker Profile
- **Access Tier:** Public.
- **Behavior:**
  - Looks up maker by `@handle`.
  - Fetches maker's public identity and list of available offers created by them in $O(1)$ query count.
- **Affordances:**
  - **Back Navigation:** "← Back to all offers" link (`/offers`).
  - **Maker Identity:** Maker's display name (`h1`), handle badge (`@handle`), and bio paragraph (omitted neatly if empty).
  - **Maker's Catalog:** Dedicated section ("Offers by {display_name}") listing all active offers created by this maker.
  - **Catalog Offer Cards:** Title links to `/offers/{slug}`, price, description, keyset pagination.
  - **Empty State:** "No offers available right now" if maker has no published offers.
- **Failure States:**
  - Non-existent handle returns HTTP 404 Not Found.

---

### 2. Authentication & Session

#### `GET /auth/sign-in` — Sign-In Portal
- **Access Tier:** Public (guests; authenticated users redirect to `/profile`).
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
  - Creates authenticated session, writes HTTP-only `sid` session cookie, marks link consumed, and redirects to `/profile`.
- **Failure States:**
  - Expired, consumed, or invalid token returns verification error view with "Request a new link" button.

#### `GET /auth/google` — Google OAuth Initiation
- **Access Tier:** Public.
- **Behavior:**
  - Generates random CSRF state token and sets in HTTP-only short-lived cookie (`oauth_state`).
  - Issues HTTP 303 redirect to Google OAuth 2.0 authorization consent endpoint.
  - If Google OAuth is unconfigured, falls back to `/auth/sign-in`. If already signed in, redirects to `/offers`.

#### `GET /auth/google/callback` & `GET /auth/google/redirect` — Google OAuth Callback
- **Access Tier:** Public.
- **Behavior:**
  - Reads `code` and `state` from query parameters.
  - Verifies state against `oauth_state` cookie (CSRF prevention).
  - Exchanges authorization code for access token via Google token endpoint.
  - Fetches user info from Google UserInfo endpoint (email, display name).
  - If user exists by email, authenticates; if user is new, provisionally assigns default handle and display name.
  - Creates authenticated session, sets `sid` cookie, and redirects to `/profile`.

#### `POST /auth/sign-out` — Session Termination
- **Access Tier:** Authenticated.
- **Behavior:**
  - Invalidates current session record from database.
  - Clears `sid` cookie.
  - Issues HTTP 303 redirect to `/auth/sign-in`.
  - Accessible via the Account section of the `/profile` dashboard.

---

### 3. Owner Dashboard & Management

#### `GET /profile` — Owner Profile Dashboard
- **Access Tier:** Authenticated (unauthenticated requests redirect to `/auth/sign-in`).
- **Affordances:**
  - **Identity Preview:** Current display name, `@handle`, bio, and email address.
  - **Public Profile Affordance:** "View Public Profile" link pointing directly to `/profiles/{handle}`.
  - **Edit Profile Affordance:** "Edit Profile" button linking to `/profile/edit`.
  - **Update Notice:** Green success banner ("Profile updated successfully") when redirected from edit with `?updated=1`.
  - **Your Offers Section:**
    - "Create an Offer" primary button (`/offers/manage/new`).
    - List of owner's offers with "Edit" (`/offers/manage/{id}/edit`) and "Delete" (`/offers/manage/{id}/delete`) actions.
    - Empty state affordance if user has no offers yet.
  - **Account & Session Section:**
    - Displays authenticated email.
    - "Sign Out" form button (POST `/auth/sign-out`).

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

#### `GET /offers/manage` — Dedicated Owner Offers Catalog
- **Access Tier:** Authenticated.
- **Affordances:**
  - "Create an Offer" button (`/offers/manage/new`).
  - List of owner's own offers with edit and delete links.
  - Keyset pagination controls.

#### `GET /offers/manage/new` — Create Offer Form
- **Access Tier:** Authenticated.
- **Affordances:**
  - Text input for `title` (3 to 100 characters).
  - Textarea for `description` (10 to 2,000 characters).
  - Number input for `price` (minor units; 0 for free).
  - Select dropdown for `currency` (NGN, USD, EUR, GBP, CAD, AUD, KES, GHS; required if price > 0).
  - "Create Offer" submit button.
  - "Cancel" link back to `/profile` / `/offers/manage`.

#### `POST /offers/manage/new` — Process Offer Creation
- **Access Tier:** Authenticated.
- **Behavior:**
  - Validates fields, generates URL slug from title and unique suffix, persists offer record associated with current user.
  - Redirects (HTTP 303) to public offer view `/offers/{slug}`.

#### `GET /offers/manage/{id}` — Owner Offer Detail View
- **Access Tier:** Authenticated (owner only).
- **Behavior:**
  - Validates that current user owns offer `{id}`. Probing non-existent IDs or IDs belonging to other users returns identical HTTP 404 (IDOR prevention).
- **Affordances:**
  - Displays offer properties.
  - "Edit Offer" button (`/offers/manage/{id}/edit`).
  - "Delete Offer" button (`/offers/manage/{id}/delete`).
  - Link to public view (`/offers/{slug}`).

#### `GET /offers/manage/{id}/edit` — Edit Offer Form
- **Access Tier:** Authenticated (owner only).
- **Affordances:**
  - Pre-filled inputs for title, description, price, and currency.
  - "Save Changes" submit button.
  - "Cancel" link back to `/offers/manage/{id}`.

#### `POST /offers/manage/{id}/edit` — Process Offer Update
- **Access Tier:** Authenticated (owner only).
- **Behavior:**
  - Applies partial patch updates while preserving untouched fields.
  - Validates domain rules and updates database.
  - Redirects (HTTP 303) to `/offers/manage/{id}`.

#### `GET /offers/manage/{id}/delete` — Delete Confirmation Screen
- **Access Tier:** Authenticated (owner only).
- **Affordances:**
  - Explanatory prompt identifying the specific offer to be removed.
  - Destructive "Delete Offer" submit button (POST `/offers/manage/{id}/delete`).
  - "Cancel" link back to `/offers/manage/{id}`.

#### `POST /offers/manage/{id}/delete` — Process Offer Deletion
- **Access Tier:** Authenticated (owner only).
- **Behavior:**
  - Enforces ownership and removes offer record from database.
  - Redirects (HTTP 303) to `/profile`.

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
  Disallow: /offers/manage/
  ```
- **Purpose:** Allows web spiders to index public content while disallowing private management areas.
