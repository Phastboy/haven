# Bounded Public diagnosis — 2026-10-30

> Release decision update — 2026-10-30: the owner accepts v0.7 for release
> with documented performance/verification debt deferred to the development
> freeze. See [release acceptance](../../../V0.7_RELEASE.md) and
> [debt register](../../../V0.7_DEBT.md). The failed/omitted measurements below
> are unchanged. Historical “blocked” statements describe the technical gate;
> they no longer prohibit the scoped release exception. No performance pass is claimed.


**Conclusion:** Public category/currency/price-filter query execution dominates
this workload. JIT is ruled out for the captured queries. No application fix or
full-gate rerun was made; v0.7 remains blocked.

## Reproducible run

[CI 41078987201](https://github.com/Phastboy/haven/actions/runs/41078987201),
commit `391d0f4`, branch `codex/public-contract-profile`. Application code is
unchanged from `4c379aa`. Four fixed 30-second guest Public stages, 200 workers,
100% semantic checks, pool size 20. Run input: `public_profile=true`; gate and
Saved profile false. Diagnostic CI success means measurements completed, not
that contract workloads passed. Each subprocess has a 90-second deadline.

Initial [CI 41076712717](https://github.com/Phastboy/haven/actions/runs/41076712717)
on `8a37c00` stopped after the first workload because the docker stats client
ignored termination. `391d0f4` adds a kill fallback; the same schedule was retried.
Its partial artifact remains at `/tmp/haven-public-41076712717`.

Retained summaries, statement costs, hardware, revision and DB container stats:
[evidence/41078987201](evidence/41078987201). Full artifact:
`/tmp/haven-public-41078987201`. GitHub artifact retention is seven days.

## Measurements

| JIT | Offered journeys/s | Achieved HTTP/s | Dropped arrivals | HTTP p95 ms | Pool acquisition p95 ms |
|---|---:|---:|---:|---:|---:|
| on | 100 | 99.99 | 0 | 45.49 | 2.43 |
| on | 1010 | 263.88 | 22,064 | 1122.22 | 452.75 |
| off | 100 | 99.99 | 0 | 45.14 | 2.07 |
| off | 1010 | 279.13 | 21,548 | 1061.78 | 410.97 |

Guest Public uses one request per journey. Every completed request and sampled
semantic case passed in all four stages; no completed request failed. Both
1,010/s workloads returned nonzero runner exits due to dropped arrivals, retained
as diagnostic evidence. Low-rate stages completed 3,000/3,000 each. These short,
sequential measurements do not establish a capacity ceiling or a gate pass.

## Attribution

The two statement shapes with `category`, `currency` and `price <=` predicates
(first/later page) consumed 98.64% of captured execution time with JIT on at
1,010/s; 98.67% with JIT off. At 100/s their mean execution times were roughly
34–41 ms, versus under 0.3 ms for the other leading feed shapes. Under load,
price-filtered mean execution grew to 298–311 ms with JIT on and 291 ms off;
mean planning was roughly 1.3–1.5 ms. The evidence localizes dominant cost to
execution of these two shapes, rather than cover retrieval or planning.

All captured statement JIT counters were zero in every phase, including JIT-on
runs. Disabling JIT did not clear the contract failure. Do not propose disabling
JIT as the fix based on this run. No causal significance is assigned to the
small difference between the two overloaded stages.

Database container CPU reached approximately 315% (about three logical CPUs);
valid samples near the overloaded stages were around 300%. Container memory
at the last valid samples was about 260–286 MiB. Pool acquisition p95 rose from
2–2.4 ms at low rate to 411–453 ms under load, consistent with costly statements
occupying connections. Pool metrics include connection creation. This identifies
a database-heavy bottleneck but does not prove a particular scan or join plan.

The Public query in `crates/haven-db/src/offers/feed.rs::list_public` joins makers
before `ORDER BY`/`LIMIT` and uses `.persistent(false)`; facet emission adds bound
currency and price constraints. No EXPLAIN plan was captured, so an index mismatch,
poor selectivity estimate or excessive join work remains to be distinguished.

## Stopping point and next step

The four-stage diagnostic is complete. Next: capture `EXPLAIN (ANALYZE, BUFFERS,
SETTINGS, FORMAT JSON)` for the first/later free-price fixture cases and fast
controls on the same seeded CI database. Inspect existing indexes and cardinality
estimates; choose one minimal query/index correction only after the plans explain
the observed cost. Verify correctness and Public contract rate before a full-gate
rerun. Do not increase pool size, lower targets, or reopen Saved diagnosis from
this evidence. No soak, spike, mixed, personal contract or external-image claims
are supported by this diagnostic.
