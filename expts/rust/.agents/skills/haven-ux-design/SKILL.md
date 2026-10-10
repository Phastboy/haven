---
name: "haven-ux-design"
description: "Use when designing or reviewing how Haven screens behave and feel: flow, feedback, consistency, icons, spacing, alignment, typography, and restraint with effects. Pairs with haven-ui-design."
---

# Haven UX design

UI design (`haven-ui-design`) decides structure, tokens and components. This skill decides how the product **behaves and feels to use**: the rules that keep screens simple, consistent and professional. Use both. Where they overlap (tokens, slots, modularity, accessibility), `haven-ui-design` wins.

Every rule here follows one idea: **a person should understand the screen at a glance and never wonder what just happened.** Every decision still needs a written reason in `docs/DESIGN.md`.

## 1. User flow and journey
- Start from the person's goal and what they just did, using `docs/JOURNEY.md`. Design only for the current stage.
- One primary action per screen. The next step is always obvious.
- No dead ends: every screen offers a way forward and a way back. After success, say what happened and what to do next. After failure, say how to recover.
- Fewer steps beat clever steps. Do not add a step, screen or confirmation without a reason tied to the journey.
- Check: from any screen, can someone say what to do next without reading everything?

## 2. Simplicity
- Prefer the simplest element that does the job: a plain link, a standard button, a normal text field. Use native HTML controls before custom ones.
- If a screen feels busy, remove before rearranging.
- Progressive disclosure: show what most people need first; put the rest one step away.

## 3. No redundant elements
Every element must earn its place. Test: **if this were removed, would anyone lose information or the ability to act?** If not, remove it.
- No repeated headings, duplicate actions, or helper text that restates the label.
- No decoration that carries no meaning (dividers, boxes, badges, icons that repeat adjacent text).
- Show data once, in the place people look for it.

## 4. Consistency (inconsistent components are a defect)
- One component per job. The same action has the same label, position, size and style everywhere.
- Same component, same internal spacing, same states (default, hover, focus, active, disabled, error).
- Before adding a component, search for an existing one that does the job. If two components are near-duplicates, merge them rather than adding a third.
- When improving a component, improve it everywhere it appears, through the component, never by patching pages.

## 5. Icons: a rule, not a mood
Icons help when they let someone grasp meaning at a glance and cost when they decorate or make people guess. Rule:
1. **Icon plus label by default** for navigation and actions.
2. **Icon-only only for this closed set** of universally understood actions, always with an accessible name and a visible focus state: **close/dismiss**, **expand/collapse**, **open/close menu**. Every other action is icon plus label. To add an action to the set, record it in `docs/DESIGN.md` with the reason; until it is recorded there, it is not in the set.
3. **No icon where text is already faster to scan:** form labels, body copy, headings, table text.
4. **No icon that repeats the label next to it** without adding meaning (redundant, see 3).
5. **One concept, one icon, everywhere.** One style, one size scale, one stroke weight. Never two icons for the same concept or one icon for two concepts.
6. Icons never carry meaning alone; the meaning is also in text or position.
- Decision test: does this icon let a person get the meaning faster than the words, for this exact place? If not, cut it.

## 6. Interactive feedback
Something visible should happen immediately, ideally instantly, after every interaction.
- Every interactive element has hover, focus, active (pressed) and disabled states, styled consistently. Pressed state responds with no perceptible delay.
- After a submission, the person sees confirmation or an error in the place they are looking, not elsewhere. Errors sit next to the field they concern.
- Server-rendered flows: respond fast (it counts against the load-testing gate), redirect after success (post/redirect/get), and show a clear confirmation. Where double submission is a risk, handle it on the server first; a client-side guard needs JS and a reason in `DESIGN.md`.
- Never leave a person unsure whether an action worked.

## 7. Effects: gated, not default
Effects read as immature and are a common AI tell unless they have a job. Default to **none**.
- Allowed: functional feedback only (focus outline, hover and pressed states, a short state change). Keep it brief, subtle and consistent, and respect `prefers-reduced-motion`.
- Not allowed without a written functional reason: gradients, glows, blurs, decorative shadows, glass effects, parallax, entrance or scroll animations, bouncing, shimmering.
- Gate: every effect in `DESIGN.md` states what job it does that structure, spacing and weight cannot. No job, no effect.

## 8. Spacing
Elements need room to breathe.
- Use only spacing tokens, never arbitrary values.
- Group by proximity: space between groups is clearly larger than space within groups.
- Same component, same padding. Consistent vertical rhythm down the page.
- If it feels crowded, fix spacing before removing content; if it feels sparse, check for redundancy first.

## 9. Alignment
- Consistent left edges and shared baselines. The same slot sits in the same place on every row (see slot-based rows in `haven-ui-design`).
- Align to a small set of edges per screen. Avoid centring body text or mixing alignments within one block.
- Check: draw a line down the left edge; most things should sit on it.

## 10. Typography
- One family (system stack by default, per `haven-ui-design`), a small type scale (about 4 to 6 sizes) and 2 or 3 weights, all as tokens. Do not add a size for a single use without a reason.
- Hierarchy comes from size, weight and spacing, so it works in black and white.
- Comfortable line length (about 45 to 75 characters) and line height for body text. Left-aligned text. Sentence case.
- Numbers people compare (prices) use tabular figures and consistent formatting.
- Define behaviour for long text: wrap or clamp (for example two lines) per component, never overflow.
- Do not use bold, caps and colour all at once to say "important"; pick one.
- Concrete values live in the tokens file; record any change there with a reason.

## Process
1. State the journey step and the single primary action.
2. Remove redundancy and unnecessary steps.
3. Check consistency against existing components before adding any.
4. Apply spacing, alignment and type from tokens only.
5. Decide icons by the rule in section 5, effects by the gate in section 7.
6. Design feedback for every interaction and every state.
7. Check the screen in grayscale and at mobile width.
8. Record decisions and reasons in `docs/DESIGN.md`.

## Review checklist (report only violations)
- Is it clear what to do next, and is there exactly one primary action?
- Is anything redundant?
- Any component that duplicates or diverges from an existing one?
- Icons: does each follow the rule? Any inconsistent or decorative?
- Does every interaction give immediate, visible feedback?
- Any effect without a stated job?
- Spacing and alignment on tokens and edges? Type within the scale?
- Does it still read in grayscale?

## Stop and ask
- A flow decision depends on a product decision that is not in the journey or schema.
- A look-shaping choice (icon set, type scale, theme) is undecided.
- Two sound designs differ by a product trade-off.
