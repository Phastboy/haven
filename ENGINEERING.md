# Haven Engineering Rules

These are the principles that govern how Haven is built.
They apply to every version, every feature, and every line of code.

---

## Reengineering

**Nothing ports merely because the old implementation had it.**

Before porting any concept, ask: why does this exist?
If the answer is good, keep the underlying requirement.
If the answer is "because the previous framework made us," question it.
If the answer is "we hadn't thought about this yet," add it to the investigation.
If the answer is "this was temporary," don't port it.

**The existing implementation is evidence, not the specification.**

Haven's requirements are the specification.
The current implementation is one solution — evidence of what has been tried.
It teaches us how to build Haven properly. It is not what we are building.

**Everything must earn its existence.**

A type, a field, a table, a status, a concept — each must justify its presence
by having observable effects within the current version.
If removing it changes nothing, it does not belong yet.

**We do not arrive at "the canonical X architecture."**

We ask whether the architecture is actually the best solution for our requirements.
Every abstraction, pattern, and tool is evaluated against Haven's actual needs.

---

## Roadmap discipline

**The roadmap is the product boundary, not the implementation backlog.**

Each version answers a question. The checkboxes define what a person must be able to
accomplish. The engineering work inside that boundary may be far larger than the
checkbox suggests.

**We may go deeper than the checkbox. We may not go wider than the version.**

We can spend significant time discovering something hidden inside one checkbox.
We cannot spend that time designing future architecture because we got excited about it.

**Depth is allowed. Scope expansion is not.**

**Treat the current version as the only world that exists.**

Nothing from a future version shapes any decision in the current version.
We can know that later versions exist, but we do not use their requirements to
justify decisions now.

**A version is not closed until it is genuinely done.**

Done means:
- The product question is answered
- The hidden engineering journey has been completed
- The load-testing gate passes

Checked boxes mean we understand the problem from first principles, have implemented
the right solution, have tested it, have measured it, and are satisfied.

---

## Investigation before implementation

Before writing any code for a feature, work through:

```
Requirements        — what must be true?
Invariants          — what can never be violated?
State transitions   — what states exist and what causes them?
Failure modes       — what can go wrong and how is each handled?
Security boundaries — what must be prevented, and by what mechanism?
Persistence         — what must survive, and what consistency is required?
Concurrency         — what happens under simultaneous access?
Observability       — what must be knowable after the fact?
Performance         — what are the targets?
Alternatives        — is this the right approach?
```

Only after this investigation do we design. Only after design do we implement.

**Find the hidden engineering journey.**

A checkpoint like "a person can sign in" may conceal:
```
request sign-in → normalize email → identify or create account →
generate credential → persist it safely → deliver it →
verify it → prevent replay → establish session →
persist session safely → authenticate subsequent requests →
establish authorization boundary
```

Those are not new features. They are the engineering work required to make one
checkbox real.

**Challenge every existing decision.**

When we encounter a pattern from the existing implementation, we ask:
- Does v0.1.x actually require this?
- Does the current version have two concepts here, or did we inherit the split?
- What behavior depends on this value?
- What breaks if we remove it?

If nothing breaks, it does not belong in this version.

---

## Domain modeling

**A concept earns its existence by having observable effects.**

If a field, status, or type has no behavioral consequence in the current version,
it does not exist in the current version.

**Premature structure is not "modeling correctly."**

Modeling a future version's concepts in the current version is not better design.
It is scope expansion disguised as foresight.

**Status machines require external observers.**

A status that affects visibility requires someone to see it.
A status that represents soft deletion requires something external to protect.
Without those conditions, the status does not exist yet.

**Validated types are constructed once.**

A type like `Email` or `Price` is constructed through a parse or constructor function
that validates it. After construction it is guaranteed valid. Any function that
accepts a validated type does not re-validate.

---

## Type system

**If two values of the same underlying type can be confused, make them different types.**

`AccountId`, `OfferId`, and `SessionId` are all UUIDs at the storage level.
They are distinct types. Passing an `AccountId` where an `OfferId` is expected
must not compile.

**The compiler enforces invariants; code review does not.**

If a class of bug can be made impossible by the type system, it must be.
This is not a style preference — it is a reduction in the class of bugs that
can reach production.

**`PlaintextToken` and `HashedToken` are different types.**

A function that stores a token takes a `HashedToken`.
A function that sends a token to a user takes a `PlaintextToken`.
It is physically impossible to store a plaintext token or send a hash to a user.

**Enum variants must be exhaustive at every use site.**

Adding a variant to an enum must surface every place that needs to handle it.
Wildcard matches on domain enums require justification.

**Schema changes must break the build.**

SQLx compile-time query verification is not optional.
If the schema changes and a query is not updated, the build fails.
A schema change that silently breaks queries is not acceptable.

---

## Database

**Business invariants that can be expressed as database constraints must be.**

If an invariant can be expressed as a CHECK constraint, a UNIQUE constraint,
a NOT NULL, or a FOREIGN KEY, it lives in the schema.
A bug in the application cannot violate a database constraint.
This is a layer of correctness that does not depend on application logic.

**Hard delete when nothing external depends on the record.**

Soft deletion (status flags, deleted_at columns) exists to protect external
references to a record. When no external reference exists, hard delete is correct.
Soft deletion without a reason is complexity without benefit.

**snake_case column names.**

PostgreSQL normalizes unquoted identifiers to lowercase.
snake_case column names map naturally to Rust field naming.
Quoted camelCase identifiers are avoided.

---

## Security

**Store token hashes, never plaintext tokens.**

Magic link tokens and session tokens are stored as `SHA-256(token)`.
The plaintext token lives only in the delivery channel (email link, cookie).
A database compromise does not yield working credentials.

**No enumeration.**

Sign-in always responds identically regardless of whether the account exists.
A request for a resource the person does not own returns the same response as a
request for a resource that does not exist.
An attacker cannot use response differences to discover what exists.

**Authorization check order is fixed.**

```
Is the person authenticated?     → No → redirect to sign-in
Does the resource exist?         → No → 404
Does the person own the resource? → No → 404
Proceed.
```

Authorization is always checked after authentication and after resource loading.
The 404 response for "not owner" is deliberate — it does not reveal that the
resource exists.

**Session cookies are HttpOnly, Secure, and SameSite=Lax.**

These are not optional settings. They are the minimum configuration for session
cookies in a production-grade system.

---

## The experiment

**The existing Haven is the control group, not the target.**

The TypeScript implementation is a working specimen.
The Rust implementation is the research implementation.
Comparing them requires that the comparison is honest — including places where
the Rust version loses.

**Don't trust magic. Understand the mechanism.**

When a framework or library does something that is not fully understood,
investigate it. Read the source. Understand the macro expansion, the wire
protocol, the runtime behavior. Only then evaluate whether the abstraction
is earning its place.

**Don't become emotionally attached to a tool.**

A tool is evaluated against Haven's requirements.
If it works well, use it.
If it works but is insufficient, augment it.
If it is the wrong abstraction, replace it.
If it is wrong for our requirements, discard it.
The goal is the best engineering solution, not any particular tool.

**If the experiment reveals that something doesn't work, that is a successful result.**

A failed experiment that teaches something is more valuable than a demo
that proves nothing.
