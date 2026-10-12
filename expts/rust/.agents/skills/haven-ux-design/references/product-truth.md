# Haven UX: Product Truth and Scope

Establishes what the product is allowed to do and what experience the current release supports.

## 1. Authority Hierarchy
Resolve conflicts in this strict order:
1. **Product Truth:** `docs/JOURNEY.md`, `ROADMAP.md`, schema, permissions, and release scope.
2. **UI Design System:** `haven-ui-design` owns visual structure, tokens, components, slots, layout, and iconography.
3. **UX Behaviour:** `haven-ux-design` defines interaction flows, feedback, and user comprehension.

Security, accessibility, and data integrity are non-negotiable. If requirements conflict, stop and ask rather than inventing a rule.

## 2. Current Product Scope Only
Implement only capabilities that exist in the current release stage:
- No speculative navigation or disabled buttons for future features.
- No placeholder sections, mock data, or fake functionality.
- Integrate new capabilities only when their roadmap stage arrives.

## 3. Data Integrity & State Truthfulness
The interface reflects server truth:
- Never imply success before the server confirms it.
- Never represent a failed request as an empty result.
- Distinguish between missing, empty, unavailable, and unauthorized resources.
- Do not rely on client hiding as the sole enforcement of permissions; the server is authoritative.
- Stop and ask when permission rules, validation constraints, or schema mappings are ambiguous.
