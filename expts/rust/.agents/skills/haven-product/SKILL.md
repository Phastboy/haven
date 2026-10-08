---
name: haven-product
description: >-
  Keeps work on Haven aligned with its product roadmap (ROADMAP.md): what stage
  the product is in, what is in or out of scope, and what the product words mean.
  Use this skill before planning, designing, or implementing any feature, route,
  schema change, or UI for Haven; when asked "should we build X", "what's next",
  or "is this in scope"; and when a task touches offers, commitment, discovery,
  search, saved items, following, agreement, or payment. Do not use it to decide
  how code is written (see ENGINEERING.md) or for dependency lookups.
---

# Haven product alignment

## Source of truth

`ROADMAP.md` at the repository root (find it with `git rev-parse --show-toplevel`). **Read it fresh every time.** This skill deliberately does not copy its contents, so there is nothing here that can go stale. The checkboxes in the file are the current state of the product.

If `ROADMAP.md` and the code disagree (a checked item that does not work, or a working feature that is unchecked), report the mismatch to the user. Do not decide which one is right.

## How the roadmap works (rules to apply)

1. **Each version answers one question.** Work should be the smallest thing that can answer the current version's question, not a collection of features.
2. **Two gates per version:** the product question is answered, **and** the system meets its load-testing target. A feature is not "done" for a version until both hold. Do not mark a roadmap item complete from a passing build alone; check the "Done when" line and the load-test status (`PERFORMANCE_CONTRACT.md`).
3. **The roadmap describes what a person can accomplish, not how.** Implementation (tables, queues, search engine, messages, payments) is deliberately left open. Do not treat any implementation detail as decided because the roadmap hints at it.
4. **Undefined means undefined.** Versions marked TBD (0.7 to 0.9 and 1.x) must not be filled in. Commitment in 0.3.x is explicitly "to be discovered and defined during this version". Agreement and closing in 1.0.x are likewise to be defined from what is learned earlier.
5. **Go deeper, not wider.** Work belongs to the **current version** (the earliest version with either unchecked items or an unmet load-test gate) unless the user explicitly says otherwise. If item completion and load-test status disagree, ask before advancing to another version. Inside that version, depth is welcome; hopping to another version's question is not. A request or a dependency that belongs to a later version (for example search during 0.1.x, or what happens to a committed offer when it is deleted) is recorded as a deferred dependency and flagged, never silently built or answered.
6. **A checkbox is never atomic.** Each roadmap item hides a sub-journey that must be untangled before it is designed or built. Use `haven-journey` for this. Detail is added just in time, only for the stage being worked on.

## Before starting any task

1. Read `ROADMAP.md`, then run the untangle step from `haven-journey` for the roadmap item (it reads and updates `docs/JOURNEY.md`).
2. State which version and which roadmap item the task serves. If it serves none, say so and ask whether it is intended (a prerequisite, a fix, infrastructure) or scope creep.
3. If the task needs a product decision the roadmap leaves open (what "commitment" means, what happens after interest, what "closing" requires, what a category is), **stop and ask the user.** Do not invent the product behaviour and do not bake an invented answer into schema or routes, since those are costly to change.
4. Prefer the smallest implementation that answers the version's question. Name what you are deliberately leaving out.

## Language

Use the roadmap's own words in code, docs, UI copy, and commit messages: **person** (not "user", where the product is meant), **offer**, **commitment**, **interest**. Do not introduce synonyms (listing, post, item, order, deal) for the same concept. If a new product term is needed, propose it to the user and record it once agreed; do not coin it unilaterally.

## What each stage implies for review

When work reaches these stages, also load `rust-security-review` and check the matching concerns:

| Roadmap stage | Concerns that become real |
|---|---|
| 0.1.x create/edit/delete offer | Ownership on every write (IDOR), CSRF on state changes, input validation |
| 0.2.x seeing offers | What is public vs private, list pagination, N+1 on feeds |
| 0.3.x commitment and communication | Authorization between two parties, who can see what, abuse and rate limits |
| 0.4.x and 0.6.x find and narrow | SQL injection through filters and sort fields, index and query performance |
| 0.5.x save and follow | Per-person scoping, unbounded lists |
| 1.0.x and 2.0.x agreement and payment | State machine integrity, idempotency, money handling, auditability |

## Proposing changes to the roadmap

The roadmap changes by what is learned, not by what is convenient. If work reveals the next question (especially for TBD versions) or shows an item is wrong, **propose** the edit to the user with the evidence. Do not edit `ROADMAP.md` on your own, and do not tick a box unless the user confirms the version's "Done when" condition is met and its load-test gate has passed.

## Report format for scope checks

```
Task: <what was asked>
Serves: <version and roadmap item, or "none">
In scope for current version: yes / no / unclear
Open product decisions: <what the roadmap leaves undefined that this touches>
Smallest thing that answers the question: <proposal>
Deliberately left out: <list>
```
