# HAVEN

> **Every version must meet its load-testing criteria before it is considered complete.**

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

The workspace package version is **0.6.0**. Stages 0.1.x through 0.6.x are complete; v0.7 implementation has begun with the behavior-preserving title/price criteria refactor. Its narrowing journeys and performance gate remain incomplete.

## 0.7.x — Can I narrow it down?

Plan: [v0.7 narrowing plan](expts/rust/V0.7_PLAN.md). Typed title/price criteria composition is implemented; existing discovery behavior is retained. Haven is deployed but unlaunched, with no real offers. The owner accepts eleven launch categories on judgement; review names/control at 390px before migration, then freeze existing slugs at launch. Real-title grouping is a post-launch review, not a migration gate. Show all categories with clear empty states; synthetic performance fixtures must include dominant, near-empty and empty categories. Category publishing, migration and feed chips passed integrated regression CI and responsive review; performance gates remain pending, so no narrowing item is marked complete. Tags are deferred until further notice by owner decision on 2026-10-28; no future version is assigned. Saved and Inquired retrieval are implemented with personal cursor binding and activity ordering, pending regression and HTTP load verification. The dense SQL spike selected indexed latest-message streaming for Inquired; the maintained-summary fallback remains available if its HTTP gate fails. The [cover patch](docs/COVER_PATCH.md) is merged independently and needs verification before launch, rather than incident treatment; sitemap discovery is a separate deliverable with its own acceptance, outside the narrowing gate. Inquiry means non-binding communication; messaging about an offer signifies no commitment.

Finding something should become more precise as the number of offers grows.

* [ ] A person can narrow offers down by category.
* Deferred until further notice: narrowing offers by tags (owner decision, 2026-10-28).
* [ ] A person can narrow offers down to things they have saved.
* [ ] A person can narrow offers down to things they have inquired about.

**The question:**

> Can I narrow down what I'm looking for?

**Done when:** A person can progressively narrow a large collection of offers to the things relevant to them.

---

## 0.8.x — TBD

**The question:**

> What is the next thing Haven needs to prove?

This version is intentionally undefined.

It should be determined by what we learn from the previous versions rather than by filling the roadmap with features in advance.

---

## 0.9.x — TBD

**The question:**

> What is the next thing Haven needs to prove?

To be defined from what Haven has taught us by this point.

---

## Future question — Can I find what's near me? (unscheduled)

The earlier proximity draft is retained as a future planning input, deliberately
outside v0.7. A person should be able to offer from a visible place, find nearby
offers and understand proximity while retaining a fully working Haven without
sharing location. The stage needs real offers in one chosen neighbourhood,
campus or business district, with a target count set before work begins.
No version is assigned here; location, privacy and ranking remain undefined.

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
                           │
                           ▼
              ┌────────────────────────────┐
  0.1.x       │ Can I offer something?     │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  0.2.x       │ Can someone see it?        │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  0.3.x       │ Can I know who is behind   │
              │ an offer?                  │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  0.4.x       │ Can someone act on it?     │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  0.5.x       │ Can someone find it?       │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  0.6.x       │ Can I come back to it?     │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  0.7.x       │ Can I narrow it down?      │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  0.8–0.9     │            TBD             │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  1.0.x       │ Can two people close it?   │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  1.x         │            TBD             │
              └─────────────┬──────────────┘
                            │
                            ▼
              ┌────────────────────────────┐
  2.0.x       │ Can Haven handle a complete│
              │ trade from beginning to end│
              └────────────────────────────┘
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
