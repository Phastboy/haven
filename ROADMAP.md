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

* [ ] A person can see available offers.
* [ ] A person can see another person's offer.

**The question:**

> Can someone see what I offered?

**Done when:** An offer created by one person can be seen by another person.

---

## 0.3.x — Can someone act on it?

Seeing an offer should be capable of leading to a meaningful response.

The first form of action we are exploring is **commitment**: expressing that you are willing to take on the offer. What exactly that commitment means, and what should happen after it, will be discovered and defined during this version.

* [ ] A person can express their willingness to take on an offer.
* [ ] The person who made the offer can know about that expression of interest.
* [ ] People can communicate when communication is necessary to understand or progress that commitment.

**The question:**

> Can someone do something meaningful about an offer?

**Done when:** An offer can receive a meaningful response from another person.

---

## 0.4.x — Can someone find it?

As the number of offers grows, people should be able to find an offer they are looking for without already knowing where it is.

* [ ] A person can find an offer by its name.
* [ ] A person can narrow offers down by price.

**The question:**

> Can I find the offer I'm looking for?

**Done when:** A person can deliberately find an offer instead of having to encounter it by chance.

---

## 0.5.x — Can I come back to what I care about?

People should be able to keep track of things and people they care about instead of having to find them again.

* [ ] A person can save an offer.
* [ ] A person can follow another person.
* [ ] A person can return to their saved offers.
* [ ] A person can return to the people they follow.

**The question:**

> Can I come back to what I care about?

**Done when:** A person's interests can persist beyond a single visit.

---

## 0.6.x — Can I narrow it down?

Finding something should become more precise as the number of offers grows.

* [ ] A person can narrow offers down by category.
* [ ] A person can narrow offers down by tags.
* [ ] A person can narrow offers down to things they have saved.
* [ ] A person can narrow offers down to things they have committed to.

**The question:**

> Can I narrow down what I'm looking for?

**Done when:** A person can progressively narrow a large collection of offers to the things relevant to them.

---

## 0.7.x — TBD

**The question:**

> What is the next thing Haven needs to prove?

This version is intentionally undefined.

It should be determined by what we learn from the previous versions rather than by filling the roadmap with features in advance.

---

## 0.8.x — TBD

**The question:**

> What is the next thing Haven needs to prove?

To be defined from what Haven has taught us by this point.

---

## 0.9.x — TBD

**The question:**

> What is the next thing Haven needs to prove?

To be defined from what Haven has taught us by this point.

---

# 1.0.x — Can two people close an offer?

An offer should be able to move beyond interest and communication into a successful conclusion between the people involved.

Exactly what "closing" an offer requires will be determined by what we learn from 0.3.x and the versions that follow.

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
              ┌────────────────────────┐
  0.1.x       │ Can I offer something? │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
  0.2.x       │ Can someone see it?    │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
  0.3.x       │ Can someone act on it? │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
  0.4.x       │ Can someone find it?   │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
  0.5.x       │ Can I come back to it? │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
  0.6.x       │ Can I narrow it down?  │
              └────────────┬───────────┘
                           │
                           ▼
              ┌────────────────────────┐
  0.7–0.9     │          TBD            │
              └────────────┬───────────┘
                           │
                           ▼
              ┌─────────────────────────┐
  1.0.x       │ Can two people close it?│
              └────────────┬────────────┘
                           │
                           ▼
              ┌─────────────────────────┐
  1.x         │          TBD             │
              └────────────┬────────────┘
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
