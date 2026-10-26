# PR #39 unresolved review plan

Read-only review plan based on the open GraphQL review threads fetched from `phastboy/haven` PR #39. All nine returned threads were unresolved and not outdated. Every thread comment was authored by `coderabbitai`. The `CHANGES_REQUESTED` review query returned no review-level comments without a thread. No code or review threads were changed by this plan.

Severity uses: **blocking** when the repository’s stated release gate or correctness contract is violated; **should-fix** for user-visible or cache/data correctness defects; **nit** for low-risk accuracy or organization issues.

## Issue 1 — Complete the public offer detail decision entry

**Thread:** `docs/DESIGN.md:223` · reviewer `coderabbitai` · `isOutdated: false` · severity **nit**.

**Issue:** The entry “Public offer detail page redesign” is numbered 13 even though decision 13 already describes the profile shell. It also sits below “Open questions,” outside the numbered architectural decisions, making references to decision 13 ambiguous.

**Code read first:** `docs/DESIGN.md` contains the existing decision 13 at the profile-shell section, decisions 18 and 19 in the architectural list, and the offer-detail entry under Open questions with a duplicate 13. Number 20 is unused.

**Possible fixes:**

1. Renumber the offer-detail entry to decision 20 and move it after decision 19. **Selected.** It preserves the existing decision taxonomy and gives the entry an unambiguous next number.
2. Move the entry to a separate offer-detail ADR and link it from the architectural decisions. This gives it a dedicated document but splits a small decision from the established list and adds a cross-reference to maintain.
3. Merge the detail-page decision into decision 19. This avoids a new number, but couples separate concerns: listing-card treatment and the single-offer view/cache contract.

**Chosen fix:** Move the whole offer-detail entry into “Architectural & route decisions” after decision 19 and number it 20. Keep its description and constraints intact. This follows the established numbered decision list and does not create duplicate or fragmented documentation.

## Issue 2 — Reconcile v0.5/v0.6 status with the load-test gate

**Thread:** `docs/JOURNEY.md:16` · reviewer `coderabbitai` · `isOutdated: false` · severity **blocking**.

**Issue:** `ROADMAP.md` states every version must meet its load-testing criteria before it is complete. Journey marks 0.5.x and 0.6.x completed while recording their load target as “not checked in this update,” so the status contradicts the repository’s release rule.

**Code read first:** The roadmap’s opening rule is explicit. Journey rows for 0.5.x and 0.6.x currently say `completed` and `not checked in this update`; the 0.7 row says `not reached`. No load-test result is recorded in those rows.

**Possible fixes:**

1. Run the approved load scenarios for both stages, record the actual results, and retain completed status only if the recorded gate passes. This yields the strongest evidence, but load testing is explicitly out of scope for this work session.
2. Keep the feature checklist items checked while changing the stage status from `completed` to `in progress — load gate pending`; update “0.7 is next” to reflect that the release gate remains open. **Selected for this plan.** It is accurate without inventing measurements or weakening the repository rule.
3. Change the roadmap rule to exempt these stages or allow completion without recorded load evidence. This would make the labels consistent by weakening the existing gate, but leaves a policy exception with no recorded decision and undermines future stage comparisons.

**Chosen fix:** Do not claim that load testing passed. Mark 0.5.x and 0.6.x as feature-complete but not stage-complete until their specified load criteria have recorded results. Keep the roadmap’s gate unchanged. Revisit completed status after the load result is available; this plan does not run k6 or any load test.

## Issue 3 — Preserve the feed cursor when returning after saving

**Thread:** `expts/rust/crates/haven-web/src/app/offers/feed.rs:174` · reviewer `coderabbitai` · `isOutdated: false` · severity **should-fix**.

**Issue:** The save control’s `return_to` preserves search and price filters but omits the valid `after` cursor. Saving from a later feed page redirects to page one, losing the user’s position.

**Code read first:** `feed.rs` decodes `query.after` into `after_cursor` before constructing `PublicFeedQuery`, which consumes that decoded cursor. The later `return_serializer` appends `q`, `currency`, `min_price` and `max_price`, then finishes without `after`. `OfferCursor::decode` validates cursor input and `encode` is available for a canonical cursor value.

**Possible fixes:**

1. Capture the original cursor string only when `OfferCursor::decode` succeeded, then append that value to the save return URL. **Selected.** It preserves the exact feed URL and excludes malformed cursors.
2. Encode the already-decoded cursor back to its canonical representation and append that value. This also preserves pagination, but adds an unnecessary decode/encode normalization step when the validated raw value is already available.
3. Have the save handler redirect to a validated same-origin Referer instead of using `return_to`. This preserves the whole originating URL but changes redirect responsibilities and requires central safe-Referer handling for this POST route.

**Chosen fix:** Retain a validated `after` value before moving `after_cursor` into `PublicFeedQuery`, append it to `return_serializer`, and keep all existing filter serialization. This uses the validation already performed in the page flow and avoids a second redirect mechanism.

## Issue 4 — Include maker updates in the public offer ETag

**Thread:** `expts/rust/crates/haven-web/src/app/offers/slug.rs:105` · reviewer `coderabbitai` · `isOutdated: false` · severity **should-fix**.

**Issue:** The rendered offer page includes the maker’s display name and profile handle, but the ETag is derived from only the offer ID and offer update timestamp. A maker profile change can leave the representation changed while an old ETag still matches, causing an anonymous conditional request to receive 304 with stale maker details.

**Code read first:** The offer route loads the maker and uses `display_name`, `handle`, avatar and joined date in the response. At the ETag call it passes only offer ID and `offer.updated_at`. Domain `User` has `updated_at`; the profile update query sets `updated_at = now()` and returns that timestamp. `cache.rs::offer_etag` currently accepts one timestamp and includes the build version.

**Possible fixes:**

1. Extend the ETag inputs with the maker’s `updated_at` and include it in the validator alongside the offer timestamp. **Selected.** Both contributing records already expose update timestamps, and profile edits update the maker timestamp.
2. Hash the complete set of maker fields used by this page into the validator. This tracks only representation inputs, but duplicates the page’s representation dependency list in cache construction and can be missed when the template adds a maker field.
3. Stop emitting/handling conditional ETags for this representation. This avoids stale 304 responses but discards the existing public conditional-cache behavior for every anonymous offer request.

**Chosen fix:** Extend the validator with the maker record’s update timestamp (using a stable fallback when the maker lookup is absent). Keep anonymous Cache-Control and 304 handling. The `User.updated_at` contract is already maintained by profile-update persistence, so this invalidates cached HTML when maker data changes without maintaining a second field-by-field dependency list.

## Issue 5 — Make the owner preview note true for private offers

**Thread:** `expts/rust/crates/haven-web/src/app/offers/slug/action_card.rs:42` · reviewer `coderabbitai` · `isOutdated: false` · severity **nit**.

**Issue:** The Owner action-card arm says the offer appears to everyone, but owner access includes private offers. That sentence is false for an offer with no public image.

**Code read first:** `view_slug` permits a non-public offer only to its owner. It classifies owners as the unit `OfferViewer::Owner`; `OfferViewer` documentation explicitly says the owner may be viewing a public or private offer. `Offer::is_public()` is based on `image_count >= 1`. `ActionCardData` currently receives a role but no visibility field, and the Owner arm renders the disputed sentence.

**Possible fixes:**

1. Replace the note with visibility-neutral copy such as “Preview of your offer.” **Selected.** It remains true for public and private owner views and needs no extra state.
2. Add an `is_public` field to `ActionCardData` and render separate public/private notes. This is precise, but couples the generic action-card component to offer visibility and requires route assembly to pass another business fact.
3. Encode visibility in an owner-viewer variant and match on `Owner { is_public }`. This keeps the match exhaustive, but expands the viewer role with a page-visibility dimension and affects every use of that role.

**Chosen fix:** Use neutral preview copy. The component has no visibility input today, and no new visibility dependency is needed to produce truthful text across both allowed owner states.

## Issue 6 — Use the detail srcset for mobile gallery slides

**Thread:** `expts/rust/crates/haven-web/src/app/offers/slug/gallery.rs:42` · reviewer `coderabbitai` · `isOutdated: false` · severity **should-fix**.

**Issue:** For the second and later photos, the `<img srcset>` selects the square-card images at mobile widths. The desktop `<source>` chooses the square variant correctly for the mosaic, but mobile does not match that source and falls through to the `<img>` srcset, producing the wrong crop/size for 4:3 slides.

**Code read first:** Each `GalleryPhoto` has a detail URL/srcset and a square tile srcset. The `<source media="(min-width: 641px)">` selects detail for the first image and square tiles for later desktop tiles. The `<img>` currently repeats that selection, so later mobile slides use the square srcset despite their 4:3 layout.

**Possible fixes:**

1. Keep the desktop `<source>` selection and make every `<img>` use `photo.srcset`. **Selected.** The source overrides it on desktop; mobile receives detail candidates for every slide.
2. Add an explicit mobile `<source>` for the detail srcset and retain an appropriate fallback `<img>` srcset. This expresses both breakpoint choices in `<picture>`, but duplicates a selection already provided by the default image and adds another source rule.
3. Remove `srcset` from the `<img>` and rely on the detail `src` for mobile while retaining the desktop source. This avoids the wrong crop but drops 2x responsive candidates on mobile.

**Chosen fix:** Use `photo.srcset` on every `<img>` and leave the desktop `<source>` unchanged. The existing `<picture>` source already switches desktop tile images; this makes the fallback/mobile source correct and retains 2x detail candidates.

## Issue 7 — Reserve mobile tab clearance, including the safe area

**Threads:**

- `expts/rust/crates/haven-web/static/css/components/nav/navigation_breakpoints.css:122` · reviewer `coderabbitai` · `isOutdated: false`.
- `expts/rust/crates/haven-web/static/css/pages/landing-responsive.css:18` · reviewer `coderabbitai` · `isOutdated: false`.
- `expts/rust/crates/haven-web/static/style.css:1538` · reviewer `coderabbitai` · `isOutdated: false`.

**Severity:** **should-fix**. These three findings share one defect: fixed mobile tabs can cover the final page content or controls.

**Issue:** The tab bar height includes `env(safe-area-inset-bottom)`, but the app-shell bottom padding is fixed at 64px. Landing and feed-specific padding rules also override shell padding, with 48px and 16px bottom clearances respectively. On devices with a bottom safe-area inset, the fixed bar can cover content.

**Code read first:** `navigation_breakpoints.css` sets `.app-bottom-tabs` to `calc(60px + env(safe-area-inset-bottom, 0px))` plus a top border, but `.app-shell-content` reserves 64px. `landing.css` sets `.app-shell-content:has(.landing-page) { padding: 0; }`, and `landing-responsive.css` gives `.landing-page` 48px bottom padding below 900px. The feed selector `.app-shell-content:has(.offers-feed)` has greater specificity than the shell base and uses the `padding: 16px 12px` shorthand at 640px and below, reducing bottom clearance to 16px.

**Possible fixes:**

1. Define one shell-level mobile-tab clearance that includes the safe-area inset; preserve that bottom padding while page selectors set only top and inline padding. Remove landing’s local mobile bottom clearance and change the feed/landing selectors so they do not overwrite the shell’s bottom padding. **Selected.** Clearance logic stays in the shell component and applies consistently to main pages.
2. Keep shell rules as-is and increase bottom padding separately in navigation, landing and feed rules. This is locally explicit but duplicates the bar height/safe-area arithmetic and can regress when another main page adds its own padding.
3. Move tabs out of fixed positioning into normal document flow and redesign the shell as a sticky footer layout. This removes overlay clearance arithmetic, but requires changing scrolling/layout behavior across every main page and is broader than the three CSS overrides.

**Chosen fix:** Centralize a mobile-tab clearance value in the shell breakpoint styles, including the safe-area inset and a small border allowance. Adjust landing and feed selectors to set their top/inline spacing without overriding shell bottom clearance. Keep the child-page override at zero because child pages do not render bottom tabs. This removes the three competing page-level clearances without changing page assembly or duplicating tab-height calculations.

## Review-level comments

The GraphQL query for `reviews(states: CHANGES_REQUESTED)` returned zero review nodes, so there are no review-level comments without a thread to include.

## Selected fixes at a glance

- **DESIGN numbering:** move the offer-detail decision into the architectural list as decision 20.
- **Journey/load gate:** keep 0.5/0.6 feature work checked but do not call the stages complete until load evidence is recorded.
- **Feed save return:** preserve a validated `after` cursor in `return_to`.
- **Offer ETag:** incorporate maker `updated_at` with the offer validator input.
- **Owner note:** change to visibility-neutral preview copy.
- **Gallery srcset:** use detail srcset on each image; retain desktop tile `<source>` selection.
- **Mobile clearance:** centralize safe-area-aware tab clearance in app-shell CSS and prevent page padding shorthands from overriding it.
