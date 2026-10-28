# Cover delivery patch

This patch repairs released v0.6 behavior independently of v0.7.

## Provenance (2026-10-28)

Commit `46e305b` introduced `CardSquare` and square feed URLs. The existing
`bin/setup_cloudinary.sh` still defined eight transformations, omitting both square
names. Read-only Admin API inspection of `dlyxyhofo` confirmed those eight exist
and permit strict delivery; `card_sq` and `card_sq_2x` return 404. This establishes
a setup omission. It does not establish whether the old script or the console
originally created the eight definitions.

The manifest preserves the verified live policy: `c_fill`, `f_webp`, face gravity
for avatars, and Cloudinary default quality (no `q` override). Square images use
320×320 and 640×640, matching the builder. No existing definition is overwritten.
The Admin API returns `format` in transformation `info`, an array of stages.
Reference: https://cloudinary.com/documentation/admin_api#transformations

## Manual environment setup and pre-deploy prerequisite

Use an authorized operator workstation. Export `CLOUDINARY_API_KEY` and
`CLOUDINARY_API_SECRET` securely; the script does not source `.env` or print API
bodies. An exported `CLOUDINARY_CLOUD_NAME` must match the explicit target.

```sh
bin/setup_cloudinary.sh --cloud dlyxyhofo --check
bin/setup_cloudinary.sh --cloud dlyxyhofo --provision
bin/setup_cloudinary.sh --cloud dlyxyhofo --check
```

Default mode is read-only. Provisioning preflights every definition, creates
missing names with strict allowance, repairs allowance on matching definitions,
and rereads each result. Conflicts, authentication failures, rate limits and
transport errors stop provisioning. Partial execution can safely be repeated.
Keep strict transformations enabled in the Cloudinary console. Keep Admin API
credentials out of CI and application rendering.

Record target cloud, manifest revision and redacted check result before each
deploy. CI can run these credential-free checks:

```sh
python3 -m unittest discover -s bin -p 'test_cloudinary*.py'
node --test crates/haven-web/static/cover-images.test.cjs
cargo test -p haven-web app::components
cargo test -p haven-web app::style
```

## Delivery fallback

Both card components retain their media dimensions, title, price and links.
On pages with cards, a same-origin deferred script captures load/error events, handles cached failures
and inspects cards inserted by the runtime. The fallback starts decorative (`aria-hidden="true"`), so loading and
JavaScript-disabled successful images do not announce a failure. Confirmed failures
hide the image and expose “Image unavailable”; successful images hide the fallback.
Offer detail pages retain their script-free layout. There are no inline event
handlers, retries, URL changes or visibility mutations. The existing CSP permits
the script. Background/text remain behind the image without JavaScript.

## CLI verification record (2026-10-28)

The owner requested lightweight CLI-only checks: no browsers or CPU-intensive
builds. Browser acceptance steps from the original plan are therefore not run.

- Cloud `dlyxyhofo`, manifest revision 1: provisioning and separate read-only
  post-check passed. The two square definitions were created; eight existing
  definitions were preserved.
- Curl GET of the originally reported private asset returned HTTP 200 for all
  four variants: `card_sq` 21,704 bytes; `card_sq_2x` 61,308 bytes; `detail`
  44,740 bytes; `detail_2x` 68,992 bytes. `file` identifies each as WebP at the
  expected dimensions. This identifies the bytes; it is not a full decoder test.
- Seven credential-free Python provisioning tests pass, including documented
  Admin API array responses, conflicts, repeat runs and post-check failure.
- CLI JavaScript fallback checks pass for cached failures/successes, lazy srcset
  failure/success and inserted cards. These use a minimal DOM fixture.
- Rust component tests passed (33) before the final test lint cleanup. The new
  test now uses fallible returns instead of unwrap/index access. Formatting and
  diff checks pass. No further Rust builds were run after the CLI-only request.
- The existing style route test requires a Topcoat asset bundle and could not
  run in the fresh worktree. A broader build exhausted temporary build space;
  its generated artifacts were cleaned up. Neither build nor lint is claimed
  as passing.
- Newly uploaded asset delivery, full image decoding and rendered page behavior
  remain unverified. The UI patch has not been deployed or released.


## PR #41 follow-up

The fallback script is scoped to non-detail pages, preserving the public detail
route's existing no-script assertion. Admin API JSON must be an object; non-object
success bodies fail with a redacted error before any provisioning write. Fallback
text starts hidden from assistive technology and is exposed only after failure.
Eight Python tests and CLI JavaScript checks cover these review fixes. The Rust
suite is left to CI under the owner's lightweight CLI-only verification constraint.
