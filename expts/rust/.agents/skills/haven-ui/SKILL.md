---
name: haven-ui
description: >-
  Designs and builds the Haven web UI (Topcoat SSR in haven-web). Covers layout,
  typography, design tokens, view components, interaction feedback, images, and
  anti-AI-tell restraint. Use when creating or editing pages, views, forms, components, or styles.
---

# Haven UI & UX design (Topcoat SSR)

## Philosophy: Restraint & clarity

The UI must feel designed with deliberate purpose, not generated from a template.
- **Reference:** Airbnb's restraint (content is decoration, minimal chrome, one accent, consistent cards, progressive disclosure).
- **Text-led until Stage 0.3.5:** Hierarchy comes from size, weight, and spacing. Design **monochrome first** (black, white, grays). If a screen fails in grayscale, the structure is wrong.
- **Anti-AI-tell restraint:**
  - Forbidden without explicit written reason in `DESIGN.md`: Gradients, glassmorphism, blurs, glows, decorative cards around everything, and motion polish.
  - No decorative icons. Use crisp inline SVG icons with visible labels (never unicode arrows like `"←"` or `"›"`). Icon-only is permitted strictly for close/dismiss and expand/collapse.
  - Plain language: Avoid marketing speak ("Welcome back!", "Seamless", "Effortlessly"). Use roadmap vocabulary (**person**, **offer**, **commitment**).

## Navigation & layout architecture

Exactly two layouts exist across the application (managed via `app::navigation`):
- **Main layout:** Top-level routes (`/offers`, `/profile`, `/`). Displays primary nav on all screen sizes. No back header.
- **Child layout:** All sub-routes (`/offers/{slug}`, `/profile/offers/*`, `/profile/edit`, `/auth/*`).
  - **Mobile (`<= 640px`):** Primary nav hidden (`.desktop-only`); header contains SVG back button with "Back" label (44px touch target).
  - **Desktop (`> 640px`):** Primary nav visible; back button stays visible beneath it.
  - **Safe back navigation:** Returns to validated same-origin `Referer`. Falls back to parent route (`/offers` or `/profile`) if referer is missing, external, or cyclical.
  - Pages never build ad-hoc navigation headers; route classification handles it centrally.

## Modularity & view slots

- **Pure view functions:** Components accept typed props and return markup. Never fetch data, read sessions, or perform authorization inside views.
- **One module per component:** Named by what it is (`offer_card`, `identity_block`, `button`), not where it is used.
- **Standard slot pattern:** [leading: icon/thumbnail] [title] [description (clamped to 2 lines)] [value: price] [status/chips] [trailing: action]. Empty slots collapse cleanly.

## Forms & interaction feedback

- **HTML first:** Semantic HTML; all interactions work without client JavaScript.
- **State mutations:** POST forms with anti-CSRF token. Follow Post/Redirect/Get on success.
- **Validation failures:** Re-render form with preserved inputs and inline errors adjacent to the invalid field.
- **Destructive actions:** Require calm confirmation step using native `<dialog popover="auto">`.
- **States:** Every interactive element has distinct hover, focus (thick high-contrast outline), pressed, and disabled states. Never leave a user wondering if an action worked.

## Images (Stage 0.3.5 & Cloudinary)

- **Placement & Aspect Ratio:** Leading slot. Landscape 4:3 for offers (1 to 5 photos); 1:1 square for profile avatar (optional, max 1).
- **Cloudinary free-tier budget:**
  - Server-side pre-shrink to 1280px max edge before upload.
  - Fixed `f_webp` delivery format. Named transformations only (`avatar`, `thumb`, `card`, `detail`).
  - Immutable public IDs (new upload = new ID). Construct all URLs exclusively through `app::components::image`.
- **Upload security:** Server-signed uploads only (no browser-direct credentials). Validate magic bytes on server (JPEG/PNG/WebP only; **strictly no SVG**). Strip EXIF location metadata before upload. Upload as private asset.
- **Visibility rule:** Offers without images are private (excluded from `/offers` feeds; returns 404 on `/offers/{slug}`; visible in owner profile with prompt to add a photo).

## Accessibility baseline

- Contrast >= 4.5:1 (3:1 for large text / focus rings).
- Visible focus indicator on all interactive controls.
- Explicit `<label>` for every input (no placeholder-as-label).
- Minimum 16px body font size to prevent iOS Safari auto-zoom.
