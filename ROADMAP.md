# HAVEN

> **Every version must meet its load-testing criteria before it is considered complete, except for an explicit owner-approved release exception.**
>
> **v0.7 exception (2026-10-30):** Product scope is accepted for release with
> documented performance and verification debt deferred to the development freeze.
> The full load gate failed; its targets are unchanged. See
> [release acceptance](expts/rust/V0.7_RELEASE.md) and [debt register](expts/rust/V0.7_DEBT.md).
> v0.7.0 is published; production deployment is not verified. This is not a performance pass.

---

## Product context

Haven is an **identity-centered trade platform** built around offers and the people behind them. It is not a general-purpose social network. An offer may be a physical good, a local service, or digitally delivered work. Profiles, follows, and inquiries support trade rather than unrelated social content.

**Identity is the frame; offer is the content.** Proximity helps people find locally relevant offers when they choose it, but does not exclude distant offers or make location a requirement for general discovery.

**Status snapshot (2026-10-30):** v0.1–v0.6 complete; v0.7.0 released under the documented exception, with maintenance deferred; v0.8 agreed but not implemented. Haven is deployed but unlaunched, with no real offers reported in the supplied roadmap. The workspace package version is **0.7.0** as recorded in that roadmap.

---

## 0.1.x — Can I offer something?

A person should be able to put something they are offering on Haven and manage it.

* [x] A person can create an account and sign in.
* [x] A person can create an offer.
* [x] A person can see their own offers.
* [x] A person can edit an offer.
* [x] A person can delete an offer.

**The question:**

> Can I offer something?

**Done when:** A person can create and manage an offer from beginning to end.

---

## 0.2.x — Can someone see it?

An offer should no longer exist only for the person who created it. Other people should be able to see what has been offered.

* [x] A person can see available offers.
* [x] A person can see another person's offer.

**The question:**

> Can someone see what I offered?

**Done when:** An offer created by one person can be seen by another person.

---

## 0.3.x — Can I know who is behind an offer?

An offer is made by someone. Haven is not a marketplace of anonymous listings competing on price; it is a place where people are known by what they offer. A person should have an identity that others can see and return to.

* [x] A person can have a profile that represents them.
* [x] A person can edit their profile.
* [x] A person can see who made an offer.
* [x] A person can visit another person's public profile.
* [x] A person's profile shows the offers they have made.

**The question:**

> Can I know who is behind an offer?

**Done when:** A person looking at an offer can find out who made it, and from there see everything else that person offers.

---

## 0.4.x — Can someone act on it?

Seeing an offer should be capable of leading to a meaningful response.

The first form of action we are exploring is **inquiry**: expressing that you are willing to take on the offer. What exactly that inquiry means, and what should happen after it, has been defined during this version as non-binding private communication between people.

* [x] A person can express their willingness to take on an offer.
* [x] The person who made the offer can know about that expression of interest, and who it came from.
* [x] People can communicate when communication is necessary to understand or progress that inquiry.

**The question:**

> Can someone do something meaningful about an offer?

**Done when:** An offer can receive a meaningful response from another person.

---

## 0.5.x — Can someone find it? (completed)

As the number of offers grows, people should be able to find an offer they are looking for without already knowing where it is.

* [x] A person can find an offer by its name.
* [x] A person can narrow offers down by price.

The load-test gate is recorded as met in [`crates/haven-web/load-tests/SCORECARD.md`](expts/rust/crates/haven-web/load-tests/SCORECARD.md).

**The question:**

> Can I find the offer I'm looking for?

**Done when:** A person can deliberately find an offer instead of having to encounter it by chance.

---

## 0.6.x — Can I come back to what I care about? (completed)

People should be able to keep track of things and people they care about instead of having to find them again.

* [x] A person can save an offer.
* [x] A person can follow another person.
* [x] A person can return to their saved offers.
* [x] A person can return to the people they follow.

The load-test gate is recorded as met in [`crates/haven-web/load-tests/SCORECARD.md`](expts/rust/crates/haven-web/load-tests/SCORECARD.md).

**The question:**

> Can I come back to what I care about?

**Done when:** A person's interests can persist beyond a single visit.

---

The workspace package version is **0.7.0**. Stages 0.1.x through 0.6.x are complete; v0.7 narrowing functionality is accepted for release under the 2026-10-30 exception. Its failed performance gate and incomplete verification remain open freeze-phase debt; v0.7.0 was published after rebase-merging PR #43; production deployment is not verified.

## 0.7.x — Can I narrow it down?

Plan: [v0.7 narrowing plan](expts/rust/V0.7_PLAN.md).
[Release acceptance](expts/rust/V0.7_RELEASE.md) and
[deferred debt](expts/rust/V0.7_DEBT.md) govern the current release decision.
Category, Saved and Inquired narrowing are implemented and correctness-verified;
the checks below record accepted product behavior. The full performance gate
failed at guest Public contract load; later stages were skipped. Performance
patches, routine bug fixes and maintenance investigations are deferred to the
post-v0.8 development freeze. No performance target is reduced or marked passed.
The existing baseline includes maintained Inquired activity and the verified Saved
fix; the ineffective Public equality experiment is excluded. Tags remain deferred
until further notice. Cover-patch launch verification and sitemap discovery retain
their separate acceptance. Inquiry is non-binding communication.

Finding something should become more precise as the number of offers grows.

* [x] A person can narrow offers down by category.
* Deferred until further notice: narrowing offers by tags (owner decision, 2026-10-28).
* [x] A person can narrow offers down to things they have saved.
* [x] A person can narrow offers down to things they have inquired about.

**The question:**

> Can I narrow down what I'm looking for?

**Done when:** A person can progressively narrow a large collection of offers to the things relevant to them.

---

## 0.8.x — Can I find what's near a place?

An explorer should be able to choose a location and discover offers ordered by their proximity to that location without losing access to general discovery.

Haven does not assume that geographical proximity matters for every exchange. Instead, it gives explorers explicit control over when proximity influences discovery.

**The principle:**

> General discovery by default. Local relevance when requested. Identity always visible.

### Core Principles

1. General discovery is the default.
2. The explorer explicitly activates proximity.
3. The explorer selects a reference location.
4. Haven ranks offers from nearest to farthest relative to that location.
5. Distant offers remain discoverable.
6. Proximity stays active until the explorer explicitly deactivates it.
7. Provider identity remains visible regardless of discovery mode.
8. Feed and search use proximity for ranking; the offer detail page provides a comprehensive breakdown of the offer's location.

### Distance Calculation

Distance is measured between the explorer's selected location and the location associated with the offer, not automatically the provider's personal location.

The explorer's selected location does not have to be their current physical location.

Proximity determines the order of discovery results, not whether an offer is eligible to appear.

### Discovery and Offer Details

**Feed and Search**

When proximity is inactive, general discovery operates without geographical distance influencing ranking.

When proximity is active, offers are ordered from nearest to farthest relative to the explorer's selected reference location.

Distant offers remain discoverable.

**Offer Detail Page**

The offer detail page provides the full available geographical context of an offer.

This separates discovery from understanding:

- Feed and search determine the order in which offers are presented.
- Offer details explain where an offer is situated and provide the location information relevant to accessing or fulfilling it.

### What Remains to Be Defined

The following questions must be resolved before implementation:

**1. Offer Location**

How geographical locations are represented for different forms of exchange, including physical goods, digital services, mobile services, and offers involving multiple locations.

**2. Missing Locations**

How offers without geographical information participate in proximity-based discovery.

**3. Location Privacy**

How much geographical information can be disclosed, particularly when individuals operate from private residences.

These are definition questions within v0.8, not reasons to expand its scope.

### The Question

> Can an explorer choose a location and discover offers ordered by their proximity to that location, without losing access to general discovery?

### Definition of Done

v0.8 is complete when:

- [ ] An explorer can explicitly activate proximity.
- [ ] An explorer can select a reference location.
- [ ] Offers are ordered from nearest to farthest relative to that location.
- [ ] Distant offers remain discoverable.
- [ ] An explorer can inspect an offer's geographical details.
- [ ] An explorer can explicitly deactivate proximity and return to general discovery.
- [ ] Proximity is never automatically activated or deactivated by the system.
- [ ] Provider identity remains visible in both discovery modes.
- [ ] Location handling and privacy requirements are satisfied.
- [ ] Existing discovery functionality continues to work correctly.
- [ ] The version satisfies its defined load-testing criteria.

**Done when:** An explorer can deliberately control geographical relevance during discovery, understand where an offer is located, and return to general discovery without losing access to offers.

---
---

## Post-v0.8 — Development Freeze & Market Validation

Once v0.8 is fully implemented, verified, and has satisfied its load-testing criteria, Haven enters a development freeze. This is a deliberate transition from building functionality to validating Haven through real-world usage.

## The Rule

**No new features or expansions of existing functionality are permitted during the development freeze.**

Development is restricted to:

1. **Corrective maintenance:** Resolving defects, security vulnerabilities, performance problems, regressions, and other issues affecting existing functionality.
2. **UI/UX improvements:** Improving the discoverability, accessibility, clarity, navigation, and usability of existing functionality.

These issues may be identified through QA, code reviews, user feedback, usability observations, production monitoring, or any other legitimate source.

The freeze restricts **what may be changed**, not how an issue is discovered.

## What Is Permitted

### Corrective Maintenance

- Bug fixes and regression fixes.
- Security patches.
- Reliability and performance corrections.
- Compatibility fixes.
- Corrections to functionality that does not behave as intended.

### UI/UX Improvements

- Making frequently used functionality easier to access.
- Improving navigation and information architecture.
- Reorganizing existing screens and interface elements.
- Improving labels, descriptions, and visual hierarchy.
- Reducing unnecessary steps in existing workflows.
- Addressing confusing or misleading interactions.
- Improving accessibility and responsiveness.
- Refining existing interfaces based on observed usability problems.

UI/UX improvements must remain within the boundaries of existing functionality.

## What Is Not Permitted

- New features or capabilities.
- Expansion of existing feature scope.
- New workflows that introduce capabilities not previously supported.
- Speculative functionality.
- Feature requests disguised as usability improvements.

## Evaluating Proposed Changes

Every proposed change should answer two questions:

1. **Does this correct a problem with existing functionality or improve how people access and use it?**
2. **Does this introduce a capability Haven did not previously provide?**

If the first answer is yes and the second is no, the change may proceed.

If the change introduces new capabilities, it must be deferred until the development freeze is explicitly lifted.

## The Principle

> Freeze feature development, not product quality.

Haven must remain responsive to usability problems discovered through real-world usage.

The purpose of the freeze is not to preserve the interface exactly as it existed at launch. It is to prevent continued feature expansion from distracting from market validation.

**Existing functionality may become easier to find, understand, and use. Haven's functional scope must remain unchanged.**

## Market Validation

The primary focus shifts from software development to market development. The objective is to validate Haven through real usage, not continued feature development.

### Priorities

1. Launch Haven into a deliberately selected community.
2. Recruit providers and encourage the creation of real offers.
3. Attract explorers and encourage meaningful discovery and inquiries.
4. Observe how people use Haven in real trading situations.
5. Identify adoption barriers, unmet needs, and actual trade outcomes.
6. Understand whether proximity improves discovery when explorers activate it.
7. Gather evidence to guide future product decisions.

### Learning Without Building

User feedback, usability observations, and new opportunities should be documented. Existing defects can be fixed and existing experiences improved under the freeze rule, but discovering an opportunity does not authorize new feature development. Record proposed capabilities for later consideration.

### Ending the Freeze

The freeze does not end automatically after a fixed period. Resuming feature development requires an **explicit product decision** informed by observed user behavior, explorer and provider feedback, real trading activity, adoption and retention patterns, and limitations found through market validation.

Until that decision is made, the freeze remains in effect.

> **Haven's next direction should be determined by what real usage teaches us, not by the desire to keep building.**

**Build what is necessary. Verify that it works. Launch it. Let the market teach us what comes next.**

---

## 0.9.x — TBD (not scheduled; subject to post-launch evidence)

Feature development is frozen after verified v0.8 completion. This version is not authorized or scheduled until an explicit post-validation product decision.

**The question:**

> What is the next thing Haven needs to prove?

To be defined from what Haven has taught us by this point.

---

## Future question — Can I find the person I'm looking for? (unscheduled)

A person should be able to find someone and return to their catalogue even when
an offer they previously saw no longer appears in discovery.

This question is recorded for later planning. Haven is not ready to take it on;
no version is assigned, and it is explicitly not planned for v0.8. Search rules
and contacting a person without an existing offer remain undefined.

---

# 1.0.x — Can two people close an offer?

An offer should be able to move beyond interest and communication into a successful conclusion between the people involved.

Exactly what "closing" an offer requires will be determined by what we learn from 0.4.x and the versions that follow.

* [ ] Two people can reach an agreement around an offer.
* [ ] An offer can move from being available to being successfully closed.
* [ ] The people involved can know what stage their interaction is in.
* [ ] A completed interaction can be recognized as complete.

**The question:**

> Can two people successfully close an offer?

**Done when:** Haven can take two people from an open offer to a mutually understood conclusion.

---

## 1.x — TBD

The 1.0 release should reveal problems that cannot be known in advance.

These versions will address those problems before Haven moves toward handling complete trades.

**The question:**

> What must Haven become capable of before it can reliably handle a complete trade?

---

# 2.0.x — Can Haven handle a complete trade from beginning to end?

Haven should eventually be capable of supporting the entire journey, including payment on the platform.

```text
Offer
  ↓
Discovery
  ↓
Interest
  ↓
Commitment
  ↓
Communication
  ↓
Agreement
  ↓
Payment
  ↓
Fulfillment
  ↓
Completion
```

* [ ] A person can offer something.
* [ ] Another person can find it.
* [ ] They can express interest.
* [ ] They can communicate and reach an agreement.
* [ ] They can pay through Haven.
* [ ] The agreed exchange can be fulfilled.
* [ ] The trade can be completed.
* [ ] Haven can reliably handle this journey at meaningful scale.

**The question:**

> Can Haven facilitate a complete trade from beginning to end?

**Done when:** A trade can happen on Haven without requiring Haven to hand the important parts of the journey off to another system.

---

# The progression

```text
HAVEN
  |
  v
0.1.x — Can I offer something? [complete]
  |
  v
0.2.x — Can someone see it? [complete]
  |
  v
0.3.x — Can I know who is behind an offer? [complete]
  |
  v
0.4.x — Can someone act on it? [complete]
  |
  v
0.5.x — Can someone find it? [complete]
  |
  v
0.6.x — Can I come back to what I care about? [complete]
  |
  v
0.7.x — Can I narrow it down? [in progress]
  |
  v
0.8.x — Can I find what's near a place? [agreed, not implemented]
  |
  v
Verification + load-testing gate
  |
  v
Development freeze (maintenance + UI/UX improvements only)
  |
  v
Launch + market validation
  |
  v
Evidence-based decision to resume feature development, if warranted
  |
  +--> 0.9.x — TBD (not scheduled)
  +--> 1.0.x — Can two people close an offer? (future)
  +--> 1.x — TBD (future)
  +--> 2.0.x — Complete trade from beginning to end (future)
```

## The rule behind the roadmap

Each version exists to answer a **question**, not to justify a collection of features.

We do not decide all the answers beforehand.

```text
Question
   ↓
Build the smallest thing that can answer it
   ↓
Observe what happens
   ↓
Learn
   ↓
Define the next question
   ↓
Repeat
```

The implementation is deliberately left out of the roadmap. Haven may use messages, orders, payments, search indexes, databases, queues, or something else entirely—the roadmap only cares about **what a person should be able to accomplish**.

**Every version has two gates:**

```text
The product question is answered
              +
The system meets its load-testing target
              ↓
        VERSION COMPLETE
```
