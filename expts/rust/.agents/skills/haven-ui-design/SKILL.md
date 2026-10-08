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

The UI should feel designed by a person who understood what each screen is for, not generated from a template. That feeling comes from **decisions with reasons**, not from a look. Reference direction set by the owner: Airbnb's restraint (content is the decoration, almost no chrome, one accent, consistent cards, progressive disclosure) with Pinterest as second reference (only relevant when saving and collections appear, roadmap 0.5.x). Offers have no images for now, so the UI is **text-led**: hierarchy comes from size, weight, color, and spacing.

Never copy a platform's surface. Take apart what it does, record what is borrowed and why it fits Haven's content, and leave the rest.

**Structure carries the meaning; color is optional.** The owner's strongest taste is alignment and coordination: a consistent structure that could hold any content, understandable in plain black and white. So design **monochrome first** (black, white, grays). Hierarchy must work from size, weight, position, and spacing alone. Add the single accent last, and only where the eye should go. If a screen stops making sense in grayscale, the structure is wrong, not the colors.

## Core rules

1. **Journey first, then screen.** Read `docs/JOURNEY.md` (via `haven-journey`). Design only screens for the **current stage and lanes that exist**. Do not design later-stage screens (for example discovery during 0.1.x).
2. **Real content, never lorem ipsum.** Take the fields, lengths, optionality, and empty cases from the schema and code that exist now (`haven-db` models, domain types). If a screen needs content the schema does not have, stop and ask; do not invent fields or copy.
3. **Every decision has a reason**, written down in `docs/DESIGN.md` (decision log: what, why, what it replaces, date). A decision with no reason is cut. Referencing an established platform is welcome when it supports the reason.
4. **JIT.** Decide a token, component, or pattern when a screen first needs it, not in advance. Do not build a component library speculatively.
5. **Modularity.** Improving one component later must not require editing the page that assembles it, and adding a component must not require editing every page. See "Structure".
6. **Verify framework facts.** How Topcoat defines view components, escapes output, serves static files, handles forms, and sets headers must be checked with `rust-crate-docs` against the locked version. Do not assume.
7. **If the owner has not decided something that shapes the look** (light or dark theme, accent color, typeface), ask. Do not choose silently and do not default to a framework's stock palette.

## Process (run per screen)

1. **Purpose.** Which stage, which lane, what is the person trying to do here, and what did they just do before arriving? One sentence.
2. **Content inventory.** List every piece of data shown or collected, with real example values at realistic lengths (short, typical, very long), and what is optional.
3. **Hierarchy.** Decide the single primary action, what the eye should hit first, second, third, and what is deliberately quiet. One primary action per screen. If two things compete, one is secondary.
4. **Reference teardown (when useful).** Name the reference screen, list what you borrow (density, grid, type scale, emphasis, interaction) and why it suits this content. A reference is not a decision until the reason is written.
5. **Layout.** Mobile-first single column with a sensible max width and comfortable line length (about 45 to 75 characters for text). Group by proximity: space between groups larger than space within groups. Use a grid only where content is genuinely tabular or repeated.
6. **Tokens.** Use only tokens from `references/tokens.md`. Add a token only with a reason, in `DESIGN.md`. **Monochrome first:** build and judge the screen in black, white, and grays, then add the accent last. Check the finished screen in grayscale.
7. **States.** Design every state before calling a screen done: empty, loading (server-rendered, so mostly "slow response"), validation error, server error, not found or not permitted, long text, many items, partially filled, disabled, focus, and failure midway. See "States".
8. **Build** as modular view components (see "Structure").
9. **Critique** against `references/ai-tells.md`. Every match must be justified by a content reason or removed.
10. **Record** decisions and open questions in `docs/DESIGN.md`.

## Server-rendered specifics

- **HTML first.** Use semantic elements (`header`, `main`, `nav`, `form`, `label`, `button`, `ul`, headings in order). Everything works without JavaScript. Add JS only when a stage needs behaviour HTML cannot give, with the reason in `DESIGN.md`.
- **Forms.** Every state-changing action is a form submission (not a link). Include the anti-CSRF mechanism (verify what Topcoat provides; see `rust-security-review`). After a successful POST, redirect (post/redirect/get) so refresh does not resubmit. On validation failure, re-render the form with the person's input preserved, an error message next to the field it concerns, and a short summary at the top for long forms. Every input has a visible `label` (not placeholder-only). Say what is wrong and how to fix it.
- **Destructive actions** (delete an offer) need a clear, calm confirmation step that names what will be removed; the destructive button is not the primary-styled one on the page unless it is the point of the screen.
- **CSS delivery.** One stylesheet served from the app's own origin, built from tokens plus per-component rules. No inline `<style>`, `style=""`, or inline `<script>`, so a strict CSP (see `rust-security-review`) keeps working. Do not load third-party CSS, fonts, or scripts. Verify how static assets are served in Topcoat and what the project already uses for CSS (do not introduce a CSS framework without asking).
- **Type.** Default to a system font stack (fast, no layout shift, no external request). A custom typeface is an owner decision, self-hosted if chosen.
- **Escaping.** All user text goes through the framework's escaping. Never use a raw-HTML escape hatch for user content. Check how the view layer does this before relying on it.
- **Performance.** Keep HTML and CSS small and avoid layout shift: these screens count against the load-testing gate. Do not add assets without a reason.

## Structure (modularity)

- **Components are small, single-purpose view functions or types** (verify Topcoat's mechanism). Each takes plain data in and returns markup; none fetches data, reads the session, or decides authorization.
- **One module per component**, with its own CSS section or file, named by what it is (`offer_card`, `field`, `form_error`, `button`), never by where it is used.
- **Pages assemble, components decide nothing about the page.** A page imports components; adding or improving a component must not require editing the other pages or a central registry beyond one export line. If improving a component forces changes to many pages, the interface is wrong.
- **Layers:** `tokens` (variables only) → `base` (element defaults) → `components` → `pages` (layout only). Lower layers never depend on higher ones.
- **Semantic token names** (`--color-accent`, `--space-4`), never raw values or color names in components.
- Components do not hard-code copy that belongs to the page; pass text in.
- **Slot-based rows and cards.** A repeated item (an offer in a list) is one component with named, optional slots in a fixed arrangement, for example: **leading** (image or icon, optional), **title**, **description** (muted, clamped to two lines), **value** (the most important secondary fact, such as price), **status** (only if a real status exists), and **action**. Alignment is fixed: the same slot is always in the same place on every row, so a list reads as one object. An absent slot collapses cleanly and the remaining slots keep their alignment, so adding an image or icon later changes the leading slot only and touches no page. Do not add a slot until the schema has data for it, and do not reserve empty space for slots that do not exist yet.

## States

For each screen record, in `DESIGN.md`, the state matrix: **empty / typical / long / error / forbidden or not found / failure midway**, and what the person sees and can do next in each. Rules:
- Empty states explain what belongs here and offer the one next action, without decoration.
- Errors say what happened and what to do, in plain words. Never show internal messages, IDs, or stack information.
- Not-permitted and not-found look the same for records the person does not own (matches `rust-security-review` IDOR guidance).
- Focus is always visible; nothing relies on color alone; interactive targets are at least 44px tall on touch.

## Accessibility baseline

Text contrast at least 4.5:1 (3:1 for large text and for UI boundaries and focus indicators), visible focus, keyboard-operable everything, labels on every control, headings in order, meaningful link text, and a design that still reads in grayscale. Check these before reporting a screen done.

## When to stop and ask

- The schema or journey does not say what a screen needs to show or collect.
- A look-shaping choice (theme, accent, typeface, density) is undecided.
- Topcoat's component, form, or asset behaviour cannot be verified.
- Two good designs differ by a product decision (for example one form vs a step-by-step flow depends on how many fields an offer really has).

## Report format

```
Screen: <name>   Stage/lane: <stage, lane>
Purpose: <one sentence>
Content used: <fields and where they come from>
Hierarchy: primary action, then order of attention
Decisions made (with reasons): ...
States covered: ...
Components added or changed: ...
AI-tells check: <matches found and how each was justified or removed>
Open questions: ...
```
