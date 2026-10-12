---
name: haven-ui-design
description: >-
  Designs and builds the Haven web UI (server-rendered Topcoat views in
  haven-web) through a decision process instead of a style: journey first, real
  content, one hierarchy per screen, reasoned design tokens, every state
  designed, modular view components, and a critique pass against "AI tells".
  Use this skill when creating or changing any page, view, form, component, or
  stylesheet in haven-web; when asked how a screen should look or be laid out;
  and when reviewing UI for quality. Not for deciding what product features
  exist (use `haven-product` and `haven-journey`) or for verifying framework
  APIs (use `rust-crate-docs`).
---

# Haven UI design (Topcoat SSR)

## Intent

The UI should feel designed by a person who understood what each screen is for, not generated from a template. That feeling comes from **decisions with reasons**, not from a look. Reference direction set by the owner: Airbnb's restraint (content is the decoration, almost no chrome, one accent, consistent cards, progressive disclosure) with Pinterest as second reference (relevant when saving and collections appear in 0.5.x).

Offers have no images yet, so the UI is **text-led until the image stage (0.3.5)**: hierarchy comes from size, weight, and spacing. Structure carries the meaning; color is optional. Design **monochrome first** (black, white, grays). If a screen stops making sense in grayscale, the structure is wrong. The primary action stands out through fill, weight, and position; errors and destructive actions carry words, icons, or placement; links are underlined; focus uses a thick high-contrast outline.

## Core rules

1. **Journey first, then screen.** Read `docs/JOURNEY.md` (via `haven-journey`). Design only screens for the current stage and lanes that exist.
2. **Real content, never lorem ipsum.** Take fields, lengths, and empty cases from the schema (`haven-db` models, domain types). Do not invent fields or copy.
3. **Every decision has a reason**, written down in `docs/DESIGN.md` (decision log: what, why, what it replaces, date).
4. **JIT.** Decide a token, component, or pattern when a screen first needs it, not in advance.
5. **Modularity.** Improving one component must not require editing the page that assembles it.
6. **Verify framework facts.** Verify Topcoat view components, escaping, and headers with `rust-crate-docs`.
7. **If the owner has not decided something that shapes the look** (theme, accent color, typeface), ask.
8. **Navigation layout rule (Main vs Child).** Exactly two layouts exist across the app:
   - **Main layout:** Top-level routes (`/offers`, `/profile`, `/`). Shows primary navigation (`.site-header`) on all screen sizes. No back header.
   - **Child layout:** Every other route (e.g. `/offers/{slug}`, `/profiles/{handle}`, `/profile/edit`, `/profile/offers`, `/profile/offers/*`, `/profile/account`, `/auth/*`).
     - **Mobile (`<= 640px`):** Primary navigation is hidden (`.desktop-only`); header contains only a clean back button (`.child-header` with an explicit SVG arrow-left icon and "Back" label, meeting the 44px touch target). Unicode character signs (e.g. `"←"`, `"›"`) are strictly forbidden as icons.
     - **Large screens (`> 640px`):** Primary navigation remains visible; the back button stays visible beneath primary navigation for consistent back-navigation affordance across devices.
   - **Safe back navigation ("Where you came from" default, Parent fallback):** The back button returns the user to the safe same-site page they came from (via validated `Referer`, enabled via `Referrer-Policy: same-origin`). If `Referer` is absent, refreshed, or external, it falls back to the child's configured parent route (`/profile` when viewing own profile, `/offers` when viewing public offers). External URLs, protocol-relative (`//`), and self-referencing loops are strictly rejected.
   - **Pages never build their own navigation headers or ad-hoc back links:** Route classification is central (`crates/haven-web/src/app/navigation.rs`); individual views assemble content only.

## Process (run per screen)

1. **Purpose & Arriving Question.** Which stage, which lane, and what question does the person arrive with? ("which of my offers is this?", "what does it cost?").
2. **Content inventory.** List real data fields shown or collected, with realistic lengths and optionality.
3. **Hierarchy & Relative Emphasis.** Decide the single primary action. Defaults stay quiet (muted gray, regular weight) so only the exception stands out.
4. **Layout & Edges.** Single column mobile-first with comfortable measure (45–75 chars).
   - **Edges:** Lock every element to a shared edge. Each row creates the edge the next stacks onto.
   - **Dividers:** Default to spacing. Hairlines are allowed only when a row needs a bottom edge to stack onto; always **inset to the content column**, never full-bleed.
   - **Cards are optional:** Default to clean sections and rows. Use a card only when a bounded unit has its own identity. Never nest bordered containers.
   - **One direction per section:** Stack vertically or scroll horizontally on narrow screens, never both.
   - **Differentiate before adding space:** Differentiate rows using real schema fields in the leading slot or status chips before adding whitespace.
5. **Tokens.** Role tokens only (`--color-bg`, `--color-surface`, `--color-text`, `--color-text-muted`, `--color-border`, `--color-focus`). Judge in grayscale.
6. **States.** Design every state: empty (first-use vs no-results), slow response, validation error, server error, not found / forbidden, long text, disabled, focus, and failure midway.
7. **Build as modular view components.**
8. **Critique against `references/ai-tells.md`.**
9. **Record decisions in `docs/DESIGN.md`.**

## Server-rendered specifics

- **HTML first.** Semantic elements only. Everything works without JavaScript.
- **Forms.** State mutations use POST forms with anti-CSRF. Post/redirect/get on success. Preserve inputs and show inline field errors on validation failure.
- **Destructive actions:** Require a clear, calm confirmation step (native `<dialog popover="auto">`).
- **CSS delivery.** One stylesheet (`/style.css`) built from tokens and semantic rules. Strict CSP (`default-src 'self'`). Zero inline styles or `<style>` blocks.
- **Type.** Default system font stack (`--font-sans`). Base body size is at least 16px (prevents iOS Safari auto-zoom on input focus).
- **Navigation on small screens:** Plain top-bar links for top-level destinations. Add a bottom bar only when there are 3 to 5 real primary destinations (JIT).
- **Deferred patterns:** Bottom sheets, swipe gestures, long-press menus, and animated transitions are deferred under the effects gate.
- **Escaping:** All user-supplied text is escaped by Topcoat.

## Structure (modularity)

- **Components are small, single-purpose view functions.** Take plain typed props in, return markup. Never fetch data, read sessions, or perform authorization.
- **One module per component:** Named by what it is (`offer_card`, `identity_block`, `profile_row`, `row_group`, `button`), never by where it is used.
- **Slot-based rows and cards:** Fixed arrangement of slots: **leading** (icon now, thumbnail later), **title**, **description** (clamped to two lines), **value** (price), **status/chips** (real categorical fields only), and **action/trailing**. An absent slot collapses cleanly without breaking alignment.
- **Images (Stage 0.3.5):** Screens are text-led until 0.3.5. Images occupy the leading slot. When current, load `haven-ui-images`.

## Accessibility baseline

Contrast ≥ 4.5:1 (3:1 for large text and boundaries/focus outlines), visible focus, keyboard-operable controls, visible labels on every form input, and designs that remain fully readable in grayscale.
