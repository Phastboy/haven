# Dense Saved investigation concluded — 2026-10-30

The measured dense Saved bottleneck is resolved at 100 journeys/s by
`dfb79ad80dd97d9fcd4b525f2afb1667f2915113`. No further exploratory replay is
planned for this issue. This does not complete the v0.7 release gate.

## Cause and fix

[Baseline CI 41042199897](https://github.com/Phastboy/haven/actions/runs/41042199897)
used PostgreSQL 17.11 and `pg_stat_statements` with planning tracking enabled.
Four Saved query shapes with price predicates accounted for 99.46% of recorded
statement execution time at 100 journeys/s. Combined category/title/price reads
touched about 143,000 shared blocks per call; Free reads touched about 35,000.
Mean execution times were 293–686 ms. Planning was about 1 ms per call and JIT
was inactive. The earlier PostgreSQL 18.6 local plans did not expose this work.

The fix uses a dependent LATERAL offer lookup for Saved queries with price
predicates. `OFFSET 0` prevents PostgreSQL from flattening the lookup and
reordering it ahead of Saved membership. Facets and visibility still apply
before LIMIT; activity order, cursor keys, ownership scope, and page size are
preserved. Category-only Saved and all Inquired queries retain their query path.

## Single verification

[Verification CI 41045410575](https://github.com/Phastboy/haven/actions/runs/41045410575)
passed all standard checks and the complete 530-case narrowing sweep (1,049
HTTP checks). The Saved profile used 100% semantic checks at each rate:

| Target journeys/s | Completed | Dropped | Baseline HTTP p95 | Fixed HTTP p95 |
|---|---:|---:|---:|---:|
| 25 | 1,500/1,500 | 0 | 59.44 ms | 3.15 ms |
| 50 | 3,000/3,000 | 0 | 93.34 ms | 2.94 ms |
| 75 | 4,500/4,500 | 0 | 119.69 ms | 3.32 ms |
| 100 | 6,000/6,000 | 0 | 1,620.93 ms | 3.07 ms |

All 30,000 HTTP checks across the four verification rates passed. At 100
journeys/s, the worst per-case full page-plus-unread journey p95 fell from
2,994.57 ms to 4.60 ms. Combined queries now touch 232–293 cached blocks per
call; Free queries touch 1,046–1,245. Their mean execution times are 0.26–0.62 ms.

The table's HTTP p95 comes from the runner's `overall` field, which counts HTTP
requests. Earlier notes sometimes called that field journey p95; full journeys
are reported separately in `journey_latencies`.

The runs used fresh synthetic fixtures on separate hosted runners with the same
workflow and PostgreSQL version. Fixture hashes differ, so this is not a replay
of identical rows on the same machine. The statement work reduction and complete
semantic verification support the fix. Profiling overhead was enabled in both.

Raw artifacts are in `/tmp/haven-saved-final-profile`,
`/tmp/haven-saved-final-verification`, and `/tmp/haven-saved-final-narrowing`.
The workflow artifacts are named `v07-gate` and `narrowing-correctness`.

## Remaining release decision

Saved is verified through 100 journeys/s (200 HTTP requests/s); this is a tested
load, not its capacity ceiling. The 1,000-rate contracts, mixed workload,
spike/recovery, soak, capacity ceiling, and external image delivery remain
unverified. Keep A4 and the v0.7 release open for those explicit gates. The
Saved bottleneck investigation itself is finished.
