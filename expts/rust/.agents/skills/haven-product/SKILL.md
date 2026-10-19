---
name: haven-product
description: >-
  Aligns Haven work with ROADMAP.md and docs/JOURNEY.md. Use before planning,
  designing, or implementing features to check current stage scope, untangle
  roadmap items into two-lane journeys (Offerer & Responder), and resolve product questions.
---

# Haven product & journey alignment

## Source of truth

- **`ROADMAP.md`** at the repository root (`git rev-parse --show-toplevel`). Read it fresh. The checkboxes in the file are the current state of the product.
- **`docs/JOURNEY.md`**: The whole start-to-end journey permitted by the product told as one continuous story with two lanes (**Offerer** and **Responder**). Handoffs occur where their paths meet.

If `ROADMAP.md` or `docs/JOURNEY.md` disagree with the code, report the mismatch to the user rather than guessing which is right.

## How the roadmap works

1. **Each version answers one question.** Work is the smallest thing that answers the current question.
2. **Two gates per version:** The product question is answered, **and** the load-test gate passes (`PERFORMANCE_CONTRACT.md`). Both must hold before marking an item complete.
3. **Go deeper, not wider.** Work strictly in the earliest version with unchecked items or unmet gates.
4. **Undefined means undefined.** TBD versions (0.7–0.9, 1.x) and unreached milestones must not be pre-built or guessed.
5. **No atomic checkboxes.** Every checkbox abstracts a sub-journey. Untangle it before writing code.
6. **No invented behavior.** If a task needs a product decision the roadmap leaves open (commitment rules, closing, categories), stop and ask the user.

## Journey untangling procedure (JIT)

When starting a roadmap item:
1. Identify the sub-steps for each active lane (**Offerer** vs **Responder**) and any handoffs between them. In 0.1.x only Offerer exists; Responder begins when the roadmap introduces visibility.
2. Ask three questions of current code/schema: what exists, what is missing, and what breaks if repeated or interrupted?
3. Identify edge states (empty, partial, interrupted, back/refresh).
4. Sort unanswered questions:
   - **Blocking:** Required for "Done when". Ask user before building.
   - **Non-blocking:** Record as assumptions; use the minimal sensible default.
   - **Deferred dependencies:** Belongs to a later stage. Flag with owning stage; do not build or design around it now.
5. Update `docs/JOURNEY.md` alongside code/schema changes for the current stage.

## Product vocabulary

Use the roadmap's precise terms in code, docs, UI, and commits:
- **person** (not "user" when product actor is meant)
- **offer** (not "listing", "post", "item")
- **commitment** (not "order", "deal")
- **interest**

Do not invent synonyms.

## Scope check report format

```
Task: <what was asked>
Serves: <version and roadmap item>
In scope: yes / no / unclear
Open product decisions: <what is undefined>
Smallest answer: <proposal>
Deliberately left out: <list>
```
