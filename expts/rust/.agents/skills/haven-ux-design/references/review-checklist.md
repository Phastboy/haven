# Haven UX: Verification and Review Checklist

Use during implementation and review. Report only actual violations and unperformed checks.

## Review Checklist
- **Journey & Purpose:** Does the screen do one job? Is the primary action visually obvious? Is the arriving question answered quickly?
- **Scope & Redundancy:** Does it serve the current release only? Is any element or copy redundant?
- **Consistency:** Are existing components reused? Are actions, labels, and slot positions consistent across views?
- **Iconography:** Are explicit SVG vector icons used? Zero unicode text signs (`"←"`, `"›"`) pretending to be icons?
- **Interaction & Feedback:** Does every interaction provide immediate feedback? Does validation preserve valid user input?
- **Effects:** Any gradients, glows, shadows, or gestures without a functional reason in `docs/DESIGN.md`?
- **Layout & Typography:** Locked to shared edges? Inset hairlines? Base body text ≥ 16px?
- **States:** Are first-use empty and no-results empty properly distinguished? Are errors calm and clear?
- **Accessibility:** Keyboard-operable? Contrast ≥ 4.5:1? Meaningful alt text on images? Does it read in grayscale?
- **Performance:** Server-rendered without client-side hydration? Preserves the 1,000 RPS contract?

## Review Report Format (Violations Only)
Keep the review report concise. Report only violations and unverified checks:

```
Screen: <name>   Stage/lane: <stage, lane>
Violations:
- Location: <screen or component>
  Rule: <rule violated>
  Impact: <what the person experiences>
  Correction: <smallest fix>

Not Verified: <checks not performed>
```
