# Design tokens (proposed defaults, each with a reason)

These are **starting proposals**, not decisions. A token becomes decided only when the owner confirms it and it is logged in `docs/DESIGN.md`. Add tokens just in time, when a screen first needs one. Components only use tokens, never raw values.

## Colors: roles, not hues

Define by **role**. Hue choices (the accent, the neutral tint, light vs dark theme) are owner decisions; ask before picking.

| Role | Token | Reason |
|---|---|---|
| Page background | `--color-bg` | Quiet base; most of the screen should be this |
| Raised surface | `--color-surface` | Only where grouping needs it; prefer spacing over boxes |
| Primary text | `--color-text` | High contrast, at least 4.5:1 on bg |
| Secondary text | `--color-text-muted` | Hierarchy without extra size; still at least 4.5:1 for readable text |
| Border / divider | `--color-border` | Separation where spacing is not enough; 3:1 where it marks a control boundary |
| Accent (single) | `--color-accent`, `--color-accent-text` | One accent for the primary action and links; scarcity is what makes it work |
| Status | `--color-success`, `--color-warning`, `--color-danger` | Meaning, always paired with text or icon, never color alone |
| Focus ring | `--color-focus` | Visible on both bg and surface, at least 3:1 |

Rules and reasons:
- **Monochromatic neutral scale plus one accent.** Fits the minimal, text-led direction; 60-30-10 is read as mostly neutral surfaces, a secondary neutral for structure, and a sparing accent.
- **Complementary or analogous schemes only with a reason** (for example a second hue to distinguish two roles on one screen).
- Name by role (`--color-danger`), never by color (`--blue`).
- If dark theme is chosen or supported, redefine the same role tokens under `prefers-color-scheme` or a theme attribute; components must not change.

## Type

| Token | Value | Reason |
|---|---|---|
| Base | 16px (1rem) | Readable default, avoids mobile zoom on inputs |
| Scale ratio | 1.25 (major third) | One fixed ratio instead of arbitrary sizes |
| `--text-sm` | about 13-14px | Captions, helper text, metadata |
| `--text-base` | 16px | Body and form fields |
| `--text-lg` | 20px | Subheadings, emphasized values |
| `--text-xl` | 25px | Page headings |
| `--text-2xl` | about 31px | Rarely; only for a genuine display moment |
| Line height | about 1.5 body, about 1.2 to 1.3 headings | Readability; tighter for large type |
| Measure | 45-75 characters | Comfortable line length for text blocks |
| Family | one system font stack | Fast, no external request, no layout shift; custom typeface is an owner decision |
| Weights | 400 and 600 (max three) | Hierarchy through weight and color, since there are no images to carry it |

Rules: at most 4 to 5 sizes in use. Hierarchy without photos comes from size **and** weight **and** color (muted vs primary), so use all three deliberately. Left-align text; center only short, single-purpose content.

## Spacing and shape

| Token | Value | Reason |
|---|---|---|
| Base unit | 8px, half-step 4px | One unit gives rhythm; the half-step covers tight spots such as label to input |
| `--space-1..` | 4, 8, 16, 24, 32, 48, 64 | Arithmetic scale; no one-off values |
| Group rule | gap between groups > gap within a group | Proximity does the grouping so boxes are unnecessary |
| Radius | small (4), medium (8), full (9999 for pills only) | Few, consistent; one radius per component class |
| Borders | 1px | Quiet separation |
| Shadows | none by default | Elevation only where something truly floats (menus, dialogs), with a reason |
| Max content width | single column about 40 to 48rem for forms and reading | Line length and calm layout |

## Components: only when first needed

Likely early (0.1.x): `button` (primary, secondary, danger), `field` (label, input, help, error), `form_error`, `page_header`, `list_row` (an offer in a list as a row, not a decorated card), `empty_state`, `confirm_destructive`. Each is introduced when a screen needs it and documented in `DESIGN.md` with its states.

## Contrast and accessibility checks (each screen)

- Text 4.5:1, large text and UI boundaries 3:1.
- Meaning never by color alone.
- Visible focus on every interactive element.
- Touch targets at least 44px.
- Screen reads correctly in grayscale.
