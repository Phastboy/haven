# Released cover-image delivery regression — independent patch

Finding, 2026-10-27: Cloudinary returns HTTP 400 with
`X-Cld-Error: Unknown transformation card_sq` and `Unknown transformation card_sq_2x`
for the reported asset in cloud `dlyxyhofo`. The immediate delivery cause is missing
named transformations, not a wrong offer public ID. Detail delivery was reported
working for the same asset. The failing feed uses `listing_card` and the central
image builder, not the older `offer_card` component.

Request IDs: `22e03d2dae8760d06500b46ad3319483` (1x),
`1bca7b766f72d0701620622c03d4861f` (2x).
Asset: `offers/623b2271-63f2-45e9-ba78-2d25ef40a50e/a06e5c7f-e37a-4c58-820a-a51fe72f05e3`.

This repairs released behavior. Use a separate branch/worktree and patch release;
it is not part of v0.7 scope, sequencing or completion gates.

## Decision: provision named transformations through the Admin API

| Option | Assessment |
|---|---|
| Versioned manifest and idempotent Admin API provisioning | Selected: preserves current URLs and strict-transformation policy; makes external configuration reproducible and drift detectable |
| Inline transformations | Removes named-definition dependency, but changes URLs and still needs allowance under strict mode; does not eliminate all external configuration |

Cloudinary documents named-transform creation and strict-mode allowance through
its [Admin API](https://cloudinary.com/documentation/admin_api), and the
[access-control rules](https://cloudinary.com/documentation/control_access_to_media).
Run provisioning manually from an authorized operator workstation during environment
setup and before deployment; do not place Admin API credentials in CI. The operator
runs check-only mode as the pre-deploy prerequisite and records the target cloud,
manifest revision and redacted result. CI may run credential-free manifest/fake-API
tests and public delivery smoke checks. Never provision during page rendering. Keep a
checked-in manifest of all builder variants, with sizes/crop/format/quality and
strict allowance. The script reads existing definitions, creates missing names,
and fails on conflicting definitions rather than silently overwriting production
transforms. Provide check-only mode, explicit target cloud and a post-provision
check. Secrets remain server-side and outside logs. Treat API rate limits and
failure as provisioning errors, not permission to weaken strict mode.

## Why it recurred: investigation still needed

Images shipped earlier, but that does not prove the newer square variants were
provisioned. Inspect Git history for their introduction; compare deployed cloud
names and environment setup, manifests/scripts, console-defined transforms and
upload presets. Record whether the failure came from a new variant omitted from
setup, cloud mismatch or manual-only provisioning. None is yet established.

## Patch steps and acceptance

1. Investigate configuration provenance and retrieve working variant definitions
   without exposing credentials. Confirm square 1x=320×320 and 2x=640×640 against
   `ImageVariant`, with crop/format/quality matching the intended image policy.
2. Implement the manifest, check/provision script and manual pre-deploy check. Test
   missing/existing/conflicting definitions and repeat runs with a fake API.
3. Provision the intended cloud, check strict allowance, then GET and decode both
   square variants for old and newly uploaded assets. HEAD alone does not verify
   image bytes. Account for cached failure responses during verification.
4. Verify browser-selected src/srcset at 1x/2x on feed, public profiles, saved views
   and detail/gallery; retain private originals and test detail variants too.
5. Record real-CDN smoke results and configuration provenance. A URL-string unit
   test cannot establish live delivery. Do not mark the patch complete before this.

## Delivery-failure state

A recorded image makes an offer visible but does not guarantee CDN delivery. On
image load/decode failure, retain the card’s square media space, title, price and
offer link; show a neutral “Image unavailable” placeholder with accessible text.
Keep the offer visible and openable, with no broken-image icon or layout collapse.
Use a CSP-compatible failure handler and a stable background/text fallback when
JavaScript is unavailable; do not retry in a loop or alter database visibility.
Patch acceptance includes blocked/400 delivery, invalid bytes and failed srcset
selection at 1x/2x, plus successful delivery with no placeholder obscuring the image.
Verify the fallback in feed, profiles and saved cards independently of CDN repair.
