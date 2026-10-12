---
name: "haven-ux-design"
description: "Use when designing or reviewing how Haven screens behave and feel: flow, feedback, consistency, icons, spacing, alignment, typography, and restraint with effects. Pairs with haven-ui-design."
---

# Haven UX design

UI design (`haven-ui-design`) decides structure, tokens, and visual components. This skill decides how the product **behaves and feels to use**: clarity, feedback, consistency, and restraint. Where they overlap (tokens, slots, modularity, accessibility), `haven-ui-design` wins. Refer to `references/product-truth.md` for authority and scope rules, and `references/review-checklist.md` for verification.

Core objective: **a person should understand the screen at a glance and never wonder what just happened.**

## 1. User Flow & Journey
- Start from the person's goal and arriving context using `docs/JOURNEY.md`. Design strictly for the current stage.
- **One screen, one job:** Settings is settings; an offer editor is an offer editor. When a distinct task arises, add a dedicated page rather than overloading the current screen.
- **People hunt, they don't read:** Answer the arriving question where the eye lands first. Judge screens by how fast that answer is found.
- One obvious primary action per screen. Relevant secondary actions remain subordinate.
- No dead ends: every screen offers a way forward and a way back.

## 2. Simplicity & No Redundant Elements
- Prefer the simplest native HTML control that does the job.
- Test: *if an element were removed, would anyone lose information or the ability to act?* If not, remove it.
- No repeated headings, duplicate buttons, or helper text that merely restates the label.
- **Do not explain harder:** When a layout feels unclear, fix the structure or visual hierarchy instead of adding tooltips or helper paragraphs.

## 3. Consistency
- One component per job. The same action carries the same label, position, size, and style everywhere.
- Search for an existing component before creating a new one. Merge near-duplicates.
- Improve shared components at their module level, never via page-specific overrides.

## 4. Icons: Explicit Vectors Only
- **Icon plus label by default** for navigation and actions.
- **Icon-only set (strictly closed):** Close/dismiss, expand/collapse, open/close menu. All others require a visible label.
- **Explicit SVG icons only:** Unicode character signs (e.g. `"←"`, `"›"`, `"✕"`) are strictly forbidden as icons. Use crisp inline SVG vector icons.
- **No decorative icons:** No icons where text scans faster (form labels, headings, body text).
- One concept, one icon everywhere. Icons never carry essential meaning alone.

## 5. Interactive Feedback
Visible feedback occurs immediately for every interaction:
- Interactive elements possess distinct hover, focus, active/pressed, and disabled states.
- Server mutations: respond fast (counts against the load test gate), redirect after success (post/redirect/get), and display confirmation at destination.
- Errors appear next to the invalid field, with a summary at top for long forms.
- Never leave a person unsure whether an action worked.

## 6. Gated Effects
Default to **none**. Effects read as amateur and constitute an AI tell unless justified by a functional job.
- **Allowed:** Brief, functional state feedback (focus outline, pressed state) respecting `prefers-reduced-motion`.
- **Forbidden without written reason in `DESIGN.md`:** Gradients, glows, blurs, decorative shadows, glass effects, animations, carousels, and gesture-only controls. Everything must work by tap/click and keyboard.

## 7. Spacing, Alignment & Typography
- **Proximity:** Space between groups is visibly larger than space within groups.
- **Edges:** Lock elements to a small set of shared edges. Comparable values (prices) share a right edge.
- **Hierarchy via scale:** System font stack (`--font-sans`), minimum 16px body text. Long text wraps or clamps to 2 lines; never overflow.
- **Relative emphasis:** Muted defaults allow meaningful exceptions to stand out.

## 8. Images (Stage 0.3.5)
- Images represent real offers or people, never decoration.
- Reserve image dimensions to prevent layout shifts.
- Imageless and failed states render the same neutral placeholder mark. See `haven-ui-images` when current.
