# Cumulative journey and endpoint contracts

B2 specification, 2026-10-29. Covers every J01–J14 group in
[the inventory](JOURNEY_COVERAGE.md). These are acceptance requirements for B3–B7,
not claims about implemented scenarios or passing measurements. Absolute targets
come from [PERFORMANCE_CONTRACT.md](PERFORMANCE_CONTRACT.md); changing a scenario
must not silently weaken them. Future stages and tags remain excluded.

## Request accounting and measurement

One journey is one execution of a named action sequence below. Every HTTP
request is counted, including expected redirects, negative cases, status checks,
and failed requests. Disable automatic redirects and follow each expected local
Location explicitly. A successful expected 303/400/401/404 is a semantic pass;
unexpected status, wrong data, timeout or dropped arrival is a failure.

Each successful authenticated HTML page with Messages navigation adds **one**
`GET /unread-status` under A2, shared by desktop and mobile indicators. The
table's `H` column counts these extra requests. Guests and auth forms add none.
An explicit status check listed in the action sequence is separate from H.
During correctness runs, wait for each action's persisted effect before checking
it. Under load, status loading runs independently of the page; report both
latencies and the page-to-status completion interval. A failed status request
must not fail page rendering; injected-failure scenarios are reported separately
from the zero-error steady gate.

A3 may reuse state across runtime navigation. For that browser variant, assert
the actual per-navigation counts and separately count mutation/return refreshes;
retain the full-navigation HTTP variant below as a reproducible upper-workload
case. Do not interpret reduced follow-up counts as additional page throughput.
No CSS/JS/image request is automatically implied by the Go HTTP client.

`N = primary requests + H`. For a fixed sequence, offering `1,000 / N`
journeys/sec offers 1,000 HTTP RPS. Fractional rates need deterministic scheduling
or a whole-number journey rate rounded **up**, with its actual offered HTTP rate
reported. For a mix, compute `HTTP RPS = sum(journey rate × requests/journey)`;
record offered/started/completed journeys, offered/achieved HTTP requests, and
drops separately. Include drain time in achieved throughput. Variable media
counts use actual issued requests; never derive throughput from journey count.

All HTML page samples include authorization, pool waiting, all repository
queries (including cover and `saved_among` batches), and serialization through
complete HTTP body receipt. Saved-status batching is database work inside the
page request, not an invented endpoint. Add pool-acquisition wait histograms and
active/waiting/max connection counts in B3/B4. Never subtract pool or unread
costs from totals to obtain a pass.

## Workload contracts

The sequences name the minimum canonical successful journey. Each assertion is
against deterministic fixture truth, not just response text or HTTP 200. `P`
means the person assigned to this worker; maker/responder/outsider actors are
distinct. Paths and query links must be taken from validated returned HTML or
fixture IDs, never guessed from sequential identifiers.

| ID | Canonical actions and expected result | Primary requests | H | Fixture and required variations |
|---|---|---:|---:|---|
| J01 magic link | GET sign-in; POST email; GET sent; GET captured verification link; GET local destination. Verify account/profile creation or existing identity and working new session. | 5 | 1 | Separate new/existing accounts; isolated mail capture; expired, reused and malformed tokens; unsafe return URL falls back locally; repeat issuance and interrupted verification. Never expose captured links or cookies in reports. |
| J01 Google | GET OAuth start; GET callback using provider adapter; GET local destination. Verify state, verified email, correct identity and session. | 3 | 1 | Success/new/existing identity; denied consent, missing/mismatched/reused state, unverified email, token/provider failure, unconfigured fallback. Count/measure outbound adapter calls separately. |
| J02 | GET new form; POST category/title/description/price/currency/idempotency key; GET owner detail; GET edit; POST changed fields/category; GET detail; GET own list; POST delete; GET deleted owner URL; GET deleted public slug. Exact values, list membership, and final 404s. | 10 | 5 | Isolated mutable offer per arrival; all accepted categories; free and paid offers; empty/partial/invalid forms; same key retried concurrently creates one ID; paginated own list; interrupted create/edit/delete. Photo publishing is J13. |
| J03 | GET feed; GET returned next page; GET one public offer; GET maker profile; GET maker next page. Check exact IDs/order, no repeats, fields and maker link. | 5 | 0 guest / 5 signed-in | Visible vs photo-less/deleted offers; first/later pages, empty catalogue, long escaped fields; guest and authenticated empty/sparse/dense histories. Unknown/deleted slug is 404. |
| J04 | GET profile; GET edit; POST valid name/bio/handle; GET profile; GET public profile; GET account. Exact persisted identity, public visibility, current account. | 6 | 5 | Own actor per worker; blank optional bio, long/invalid/duplicate handle/name, escaped markup, retry and partial input. Negative POSTs preserve entered values and do not mutate persisted identity. |
| J05 | GET public offer; GET its maker link; GET catalogue next page. Maker identity agrees on all pages; exact visible offer IDs/order. | 3 | 0 guest / 3 signed-in | At least 21 visible offers, 1 photo-less offer, missing maker, empty catalogue, long display name/bio, independent signed-in visitor. |
| J06 | POST inquiry from Responder; GET maker unread state; GET returned responder thread; GET maker owner-detail inquirers; GET maker inbox. One pair conversation, correct sender/tag/body, correct inquirer and latest activity, unread maker before reading. | 5 | 3 | Fresh pair and repeated offer inquiries within an existing pair; unrelated prior conversations; self-inquiry 400, guest redirects, missing/private offer 404, outsider denied. An inquiry remains non-binding. |
| J07 | GET inbox; GET thread; POST reply; GET peer thread; GET older messages; GET next inbox page. Check sender/body/tag, chronological thread order, reverse-activity inbox, stable cursors, correct unread/read transitions. | 6 | 5 | Empty/one/many threads and >50 messages; latest sender is self/other, equal timestamps with ID ties, tagged/untagged/deleted-offer messages; outsider/nonexistent thread and send both 404; malformed cursor and blank/overlong message rejected. |
| J08 | Open recipient SSE; POST peer message; GET recipient unread state; GET recipient thread; GET recipient unread state; reconnect SSE. Assert one delivered event with persisted ID and sender, true then false status, authorized reconnect. Close both streams. | 6 | 1 | Isolated pair; empty and dense read history, concurrent send/read, multiple messages, disconnect/reconnect, session expiry and outsider SSE. A2 delayed/failed status must leave HTML usable and unknown dot hidden. A3 adds request-count/browser refresh assertions. |
| J09 | GET combined title/price feed; GET next link; GET the retained first-page URL (browser Back); GET free-only feed; GET empty-result query. Exact fixture IDs/order; criteria retained in every link. | 5 | 0 guest / 5 signed-in | Title-only, minimum/maximum/both prices, exact boundaries, zero, each currency, Unicode/case, invalid numeric/currency input, no results, back/refresh and invalid cursor. |
| J10 | POST save twice; GET saved page and next page; POST unsave twice; GET saved page; GET public detail. One save, stable original save time, correct save state/membership, successful removal. | 8 | 4 | P has 20 existing saves plus target; separate empty/sparse/dense people; own/missing/private/deleted offer, guest and invalid action, repeated writes. Profile saved placeholders and discoverable saved scope have different visibility rules; assert both. |
| J11 | POST follow twice; GET following and next page; POST unfollow twice; GET following; GET target profile. One relationship, original order retained, correct button state and removal. | 8 | 4 | P follows 20 people before target; empty/sparse/dense sets; self/missing/guest denial; escaping, partial interruption/retry; independent target/actor per worker. |
| J12 | GET filtered public feed; GET next; GET Saved with same facets; GET next; GET Inquired with same facets; GET next. Exact scope membership/order and correct next-page decisions. | 6 | 6 | Authenticated P with >20 matching rows in each scope; separate broad/selective/empty facets, first/later page, zero/sparse/dense histories, dominant/near-empty/empty categories. Verify chips, All and Clear filters links; changed scope/facets/person cannot reuse a cursor silently. Replies/self/mismatched pairs do not qualify for Inquired. |
| J13 offer photos | POST first photo; GET owner detail; POST second photo; POST reorder; GET public detail; POST delete one; GET owner detail. Correct public cover/order, one remaining image and protected last-image rule. | 7 | 3 | Isolated image-provider adapter; deterministic valid JPEG/PNG/WebP, original over 1280px and rotated/EXIF images; invalid magic/SVG/oversize/too many photos/outsider/reorder/delete. Add explicit cover/gallery requests below. No production provider writes. |
| J13 avatar | POST avatar; GET profile; GET public profile; POST avatar delete; GET profile. Identity and avatar URL/version update and then clear. | 5 | 3 | Optional/empty avatar, valid/invalid image inputs, replacement, wrong owner, adapter failure, interrupted upload. Provider deletion/outbox effects asserted in isolated storage. |
| J14 | GET account with valid session; POST sign-out; GET protected account using former cookie; GET unread using former cookie. Correct identity, cleared/invalidated session, local auth redirect and unread 401. | 4 | 1 | Fresh mutable session per arrival, distinct other-person valid session, expired/malformed/absent cookie, repeated sign-out, concurrent account switch; no private cached status/body leaks. Additional access matrix below. |

Request counts exclude deliberate negative variants, which have their own named
case and expected count. A case that follows an additional redirect or renders
an additional page must declare and account for it. An endpoint that changes its
successful response shape requires updating this specification and fixture truth
before accepting a new baseline.

## Common access and empty-state matrix

For each protected read/write route in the inventory, issue one request per
applicable identity: owner/participant, other signed-in person, no session,
invalid session and expired session. Query/path/body person IDs must never
override the session. Missing and unauthorized private resources use the same
404 behavior; private unread returns 401 for absent/invalid/expired identity.
Redirecting HTML routes retain their current sign-in behavior. Assert no state
change after denied writes and private/no-store on private data, including error
responses. Verify current state-changing request protections with missing/invalid
origin/token cases appropriate to each route; do not invent a passing CSRF check
from status alone.

Empty variants run before seeding the worker's collection. Sparse means 1–20
relationships/messages; dense means at least 6,484 conversations and 10,000
saved/inquired offers for focal actors. Include >20 feed/inbox rows and >50
messages for paging, repeated offer messages and replies, deleted/private offers,
and tied timestamps with stable UUID order. Use a minimum global catalogue of
10,000 people and 100,000 offers, with dominant, near-empty and empty categories.
Expected IDs/order are independently derived from seeded truth, not copied from
the endpoint under test. Record fixture generator revision, seed, dataset counts
and a redacted manifest hash.

## Endpoint and tier requirements

Every finite application endpoint named above gets an isolated open-arrival
load case at **at least 1,000 HTTP RPS** for 60 seconds after warm-up: p95 ≤200 ms,
p99 ≤500 ms, zero unexpected errors, no dropped arrivals and 100% scheduled
semantic checks passed. Include each material database-work stratum (dense vs
sparse, broad vs selective vs empty, first vs later pages), rather than hiding
a slow class in an aggregate. Report 100% semantic correctness runs first, then
deterministic stratified sampling of at least 10% per case during sustained load;
every case must have nonzero successful checks. Check persisted write round trips
for every completed write journey. Resource telemetry is mandatory in both.

External mail/OAuth/image operations use local isolated adapters with fixed
responses and report request count/latency. Adapter results prove application
capacity only; real provider/CDN delivery has separate integration evidence and
must never be called a passing application endpoint tier. For each rendered
media variant issue a GET to isolated delivery storage and verify status, format,
dimensions and URL/version; report these HTTP counts/latencies separately from
the application's 1,000 RPS. Browser checks cover image failure/unknown unread
states and accessibility. The mock must not omit application decoding,
resizing/validation, signing or persistence work.

For SSE, measure connection-to-response-headers latency against the finite
request latency limits above; do not treat stream lifetime as HTTP latency.
Hold authorized streams during the messaging mix, count active connections,
events sent/received and reconnects, and verify event content/ordering. Event
receipt uses the runner's explicit 30-second correctness deadline; that deadline
is not a relaxed HTTP latency target. End every stream during drain and verify
connections/RSS return to baseline. Connections/sec and events/sec are separate
metrics from HTTP requests/sec.

The cumulative mixed workload schedules **at least 1,000 HTTP RPS** for 5 minutes
and for the full 30-minute soak. Allocate the offered HTTP requests as follows:
J01 5%, J02 10%, J03 20%, J04 5%, J05 5%, J06 5%, J07 10%, J08 5%, J09 10%,
J10 5%, J11 5%, J12 10%, J13 3%, J14 2% (100% total). Within multi-variant
groups split allocation evenly; rotate fixture strata deterministically. Weight
requests including H, not journeys. This is a declared synthetic mix, not a
claim about real traffic. All J01–J14 must run; unavailable adapters/scenarios
make the cumulative gate incomplete, never implicitly excluded or passed.

Required stages retain the absolute contract:

| Stage | Offered HTTP workload | Required result |
|---|---|---|
| Correctness | Every case at low load, all responses checked | Exact state/IDs/order/permissions; zero unexpected outcomes; finite counts agree with the declared sequence |
| Per endpoint | Each endpoint and material stratum ≥1,000 RPS ×60s | p95 ≤200ms, p99 ≤500ms, zero errors/drops, semantic checks present/passing |
| Mixed | Declared mix ≥1,000 RPS ×5m | Same latency/error/correctness gates; per-endpoint results required |
| Stress | Declared mix ≥1,500 RPS ×5m | Observe saturation and record results, distinct from steady compliance |
| Spike/recovery | Mix at 100→2,000→100 RPS, each 60s | Spike p95 <1,000ms, errors <1%, no crash/persistent failure; recovery p95 <100ms, errors <0.1%; drops count as missed offered load |
| Soak | Mix ≥1,000 RPS ×30m | Steady gates throughout; no unbounded RSS/connection growth or latency drift; meaningful CPU headroom, measured per interval |
| Capacity | Mix at contract ladder, 2m/step, extend beyond 5,000 until breach | Stop at p95 >250ms, errors >1% or resource exhaustion; record last passing and first failing stage; ≥1,000 RPS minimum sustainable capacity; verify 60s recovery at 100 RPS with spike-recovery limits |

Saturation during discovery is expected evidence, not an automatic successful
capacity gate. A ceiling below 1,000 RPS or failed recovery fails delivery.
Regressions against a better accepted comparable-hardware result remain B6's
additional gate; absolute limits alone do not authorize replacing that baseline.

## Isolation and evidence handoff

Use a fresh database per workflow/measurement job and a deterministic fixture
reset before each independent tier. Each mutating worker owns its sessions,
offers, pair peers, captured auth tokens and storage IDs. Never run sign-out or
profile mutations on read-only fixture actors. Preallocate disjoint actor pools
for concurrent arrivals; exhaustion is a fixture failure, not permission to
reuse mutable actors. Record warm-up/drain/reset durations outside timed tiers.

B3 must implement these named sequences, HTTP/body assertions, negative variants,
adapters, pool metrics and manifests. B4/B5 must report per-case request/check
counts, offered/achieved RPS and journeys/sec, p50/p95/p99, errors/drops, fixture
and revision IDs, CPU/RSS/pool/connection telemetry, and recovery/capacity results.
Mark every skipped, failed or canceled stage explicitly. Cookie/token/captured
verification links remain private and are never uploaded. B6 must enforce both
absolute and accepted regression gates in actual merge/deployment controls.

This specification completes definition work only; no cumulative performance
pass or v0.7 release completion follows from its existence.
