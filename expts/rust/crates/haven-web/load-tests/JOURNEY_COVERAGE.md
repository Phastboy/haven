# Cumulative journey coverage inventory

Track B1 of [the execution checklist](../../../PERFORMANCE_AND_JOURNEY_CI_TASKS.md).
Audited 2026-10-29 against repository `docs/JOURNEY.md`, current route modules,
the Go runner, fixture generator, and `haven-rust-ci.yml`. This is a source audit,
not a passing load report. Stages 0.1–0.7 are supported; tags and future stages
are excluded. External mail, Google, and image-provider round trips require
isolated test adapters; production services must not receive synthetic load.

## Inventory

“Partial” means the Go runner exercises some actions or checks, not that the
whole journey is verified. Rust route/repository tests do not substitute for
the cumulative HTTP journey gate. Paths describe current code; historical
`/offers/manage` references in the story have become `/profile/offers`.

| ID / stage | Lane and supported journey | Important endpoints | Existing Go coverage | Missing cumulative coverage |
|---|---|---|---|---|
| J01 / 0.1 | Both: account creation, sign-in, return intent | `/auth/sign-in`, `/auth/sent`, `/auth/verify`, `/auth/google`, `/auth/google/callback` | `warmup` GETs the sign-in form; fixtures inject sessions | Magic-link issuance/consumption, expiry/reuse, new vs existing account, Google callback success/failure, safe return intent and session rotation; deterministic mail/OAuth adapters |
| J02 / 0.1 | Offerer: create, inspect, edit, delete own offer | `/profile/offers`, `/profile/offers/new`, `/profile/offers/{id}`, `/edit`, `/delete` | `steady` write branch performs create/read/edit/read/delete/404; `authorization` checks concurrent idempotency | Exact persisted fields/category, validation and interruption/retry, own-list paging, deleted public slug, session isolation; create payload currently omits required v0.7 category |
| J03 / 0.2 | Responder: discover offers and open one | `/`, `/offers`, `/offers/{slug}` | `steady` reads feed/home; `v07` checks feed IDs/order; `v06` saves by slug | Public detail field/maker assertions, missing/deleted offer, guest vs authenticated responses/cache headers; authenticated Public feed is absent from v0.7 cases |
| J04 / 0.3 | Both: represent identity and edit profile | `/profile`, `/profile/edit`, `/profile/account` | None | Profile round trip, invalid/duplicate handle, field escaping, account/session display, permissions and interruption |
| J05 / 0.3 | Responder: identify maker and visit their catalogue | `/offers/{slug}`, `/profiles/{handle}` | `v06` follows a handle, without validating the catalogue | Maker handoff, expected identity, catalogue IDs/order/paging, empty/missing profile, long fields |
| J06 / 0.4 | Responder → Offerer: inquire about an offer | `/offers/{slug}/message`, `/profile/offers/{id}`, `/conversations/{id}` | None; fixture messages seed Inquired membership only | First/repeated inquiry, single conversation per pair, offer tag and sender identity, maker inquirers, deleted offer, self-inquiry denial, anonymous return intent |
| J07 / 0.4 | Both: inbox and private conversation | `/conversations`, `/conversations/{id}`, `/conversations/{id}/messages` | None | Exact inbox activity order/cursors, previews/latest sender, empty inbox/thread, thread paging, tagged/untagged messages, read-after-write, escaping and authorization |
| J08 / 0.4 | Both: live conversation and unread navigation | `/conversations/{id}/events`, authenticated page shells; A2 endpoint pending | None | SSE event identity/order, disconnect/reconnect, unread transitions after send/read, latest-own-message semantics, desktop/mobile dots, slow/failed independent loading and refresh counts once A2/A3 exist |
| J09 / 0.5 | Responder: title and price discovery | `/offers?q=…&currency=…&min_price=…&max_price=…`, `after` | `v05`: combined filters, next link preservation, Free and empty text; `v07`: exact fixture IDs/order | Invalid/boundary input and back/refresh; connect UI controls/links to expected results rather than only prepared URLs |
| J10 / 0.6 | Responder: save, return, unsave | `/offers/{slug}/save`, `/profile/saved`; feed saved-status database batch | `v06`: duplicate save/unsave, saved pagination; `v07`: saved scope IDs/order | Exact saved membership after each transition, empty state after removal, anonymous/own/deleted cases, saved-status batch assertions and query accounting |
| J11 / 0.6 | Both: follow, return, unfollow | `/profiles/{handle}/follow`, `/profile/following` | `v06`: duplicate follow/unfollow and pagination | Exact identity/order, empty state after removal, self/missing/anonymous cases, actor isolation across other scenarios |
| J12 / 0.7 | Offerer: publish/edit category; Responder: narrow progressively | `/profile/offers/new`, `/profile/offers/{id}/edit`, `/offers?category=…`, `scope=saved`, `scope=inquired` | `v07`: deterministic public/saved/inquired cases, sparse/dense histories, first/later pages and expected IDs/order | Category write/read round trip, chip/All/clear-filter navigation, scope/cursor tampering, authenticated Public with sparse/dense unread history, zero-history people and interrupted navigation |
| J13 / released media | Offerer: offer photos and avatar; Responder: cover/gallery viewing | `/profile/offers/{id}/images`, `/images/delete`, `/images/reorder`, `/profile/avatar`, `/profile/avatar/delete`, public image URLs | None | Upload/reorder/delete ownership and validation, cover selection, gallery/thumbnail correctness, image failure fallback; image-provider test adapter and separate live CDN verification |
| J14 / all supported stages | Both: sign-out and protected access | `/auth/sign-out`, protected profile/messaging routes | `authorization`: other-owner and anonymous offer edit/delete; idempotent create | Session invalidation, expired/malformed session, no cross-account unread cache after A3, conversation/SSE/media/profile ownership, private response headers, state-changing request protection |

All J01–J14 belong in the cumulative coverage manifest. A row may have multiple
scenarios; one load workload is not proof that every row passed. Render-only
accessibility/responsive checks and real provider integration checks need separate
evidence links, since the Go client does not execute a browser.

## Source evidence and constraints for B2/B3

- `go/cmd/haven-load/runner.go` dispatches only `smoke`, `warmup`, `steady`,
  `authorization`, `v05`, `v06`, `v07`, and the spike/capacity profiles. There is
  no communication or account lifecycle scenario.
- `journeys.go`: `steadyJourney` mixes single reads with a multi-request write
  journey. Its rate therefore does **not** equal HTTP RPS. The contract's
  historical assertion that every steady journey is one request is stale.
  Record offered journeys/sec and achieved requests/sec separately.
- `offerWriteJourney` does not supply `category` on create. Validate/fix this
  before calling the cumulative write journey functional for v0.7.
- `v06Journey` reserves 22 people per worker. Other scenarios choose sessions
  by arrival index. Reserve disjoint actor groups across scenarios, including
  the two ends of a conversation, so mutation never invalidates fixture truth.
- `narrowing.go` enforces guest tokens for Public and authenticated tokens for
  personal scopes. It checks exact offer slugs/order and pagination against a
  private fixture, with deterministic semantic sampling. Add a distinct
  authenticated Public stratum rather than treating current Public measurements
  as authenticated measurements.
- `load-test-support/src/narrowing/history.rs` seeds repeated qualifying offer
  messages and owner replies, then marks the focal person read. It does not
  exercise HTTP messaging, unread transitions, or zero-history authenticated
  people. Dense read histories must remain in the fixture after A1.
- The Go HTTP client does not execute JavaScript, fetch CSS/images, or issue
  follow-up unread runtime calls automatically. B2 must specify these
  requests explicitly and account for them in request counts and latency. Saved-status
  batches currently run inside feed rendering (`saved_among`), so measure their
  database cost within the parent request rather than inventing an HTTP route.
- `v07-gate.py` currently stops at the first failing stage, without an explicit
  report entry for all skipped stages. It samples process CPU/RSS, but does not
  establish DB pool waits, full endpoint coverage, accepted regression baselines,
  minimum capacity enforcement or measured runner cost.
- `haven-rust-ci.yml` runs ordinary checks for scoped pushes to `main` and PRs;
  sustained gates require manual `load_gate` dispatch. Its profiles are v0.7
  narrowing workloads, not all historical journeys. B4 still needs a dedicated
  every-push cumulative workflow and B6 needs actual merge/deploy enforcement.

## Handoff to B2

For every ID, define exact action order, expected fields/IDs/state, request count
(including follow-up calls), actor ownership, repeat/interruption cases, fixture
size and reset procedure. Include empty, sparse, dense and unauthorized cases as
applicable. Reuse the absolute targets in `PERFORMANCE_CONTRACT.md`: steady
≥1,000 HTTP RPS, p95 ≤200 ms, p99 ≤500 ms, zero errors; the full spike/recovery,
30-minute soak and capacity ladder remain required. Per-journey offered rates
and endpoint allocations are **pending B2**, not inferred from today's results.

B1 source audit is complete at `81f5306`, with
[workspace CI](https://github.com/Phastboy/haven/actions/runs/40837141401) passing
at `9c50d0e`. This completes the inventory only. No full cumulative journey pass
is claimed; B2–B7 remain pending.
