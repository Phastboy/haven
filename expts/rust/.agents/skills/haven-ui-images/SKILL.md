---
name: "haven-ui-images"
description: "Use when adding or reviewing images in Haven: offer and profile photos, Cloudinary upload and delivery, thumbnails, alt text, image states, and image security."
---

# Haven images (Cloudinary, SSR)

Loaded only when the image stage is current (planned as 0.3.5; confirm in `ROADMAP.md`) and the schema has image data. `haven-ui-design` decides structure, tokens, and components; `haven-ux-design` decides behaviour and feel. This skill holds everything specific to images. Where they overlap, `haven-ui-design` wins. Until the image stage is current, none of this is built (JIT).

Images are the first content that brings color, user-supplied files, a third-party host, and real bytes on the wire, touching layout, security, performance, and accessibility at once.

## 1. Limits & Placement
- **Offers:** Minimum 1, maximum 5 images. An offer without an image is not public (see Visibility). Limits are defined as named constants in domain types.
- **Profiles:** Optional; at most 1 (avatar). A second is added only when its job is defined (JIT).
- **Cover image:** The first image of an offer by position.
- **Leading slot:** Images always occupy the leading slot of a row or card.
  - Rows use small thumbnails for owner scanning/management (text and price dominate).
  - Cards use image-first layouts for responder browsing (visual recognition dominates).
- **Uniform aspect ratios:** Landscape 4:3 cropped-to-fill for offers; 1:1 square for profile avatars.
- **No text on images:** Title and price sit beside or below the image to guarantee contrast.
- **One radius:** Same corner radius token as cards. No carousels, lightboxes, or hover swaps.

## 2. Cloudinary Architecture & Free-Tier Budget
Haven never serves image bytes directly. All delivery flows through Cloudinary CDN:
- **Free tier constraint (25 credits / ~25k transformations monthly):**
  - **Server-side pre-shrink:** Cap longest edge at 1280px and re-encode to clean JPEG or WebP before upload. This also strips location metadata.
  - **Fixed delivery format:** Use `f_webp` explicitly rather than `f_auto` to avoid multiplying derived image counts across different user agents.
  - **Named transformations only:** Allow-list a small set of named transformations (`avatar`, `thumb`, `card`, `detail`) in Cloudinary and enable Strict Transformations.
  - **Fixed dimensions:** At most two widths (1x and 2x) per variant:
    - `avatar`: 96, 192 (1:1)
    - `thumb`: 120, 240 (4:3)
    - `card`: 320, 640 (4:3)
    - `detail`: 640, 960 (4:3)
  - **Derive on first request:** Generate derived sizes on first read, not eagerly during upload.
  - **Immutable public IDs:** Never overwrite a public ID; replacement always generates a fresh random ID.
- **Centralized URL module:** A single component (`app::components::image`) constructs all Cloudinary URLs. No page or other component assembles image URLs directly.
- **Performance:** Image reads hit Cloudinary CDN and do not count against haven-web's 1,000 RPS gate. Image uploads (multipart POST) hit haven-web, are rate-limited, and tested separately.

## 3. Security & Upload Pipeline
Uploads are untrusted user input:
- **Server-signed uploads only:** Never expose API secrets or use unsigned browser-direct presets. Uploads go to haven-web first, then to Cloudinary via signed API calls.
- **Allowlisted formats:** JPEG, PNG, WebP only. Strictly **no SVG** (executable script risk). Inspect magic bytes on the server, not client-supplied MIME types.
- **Strip EXIF location metadata:** Strip GPS and device metadata on the server before bytes leave haven-web. Apply EXIF orientation first so mobile photos do not appear rotated.
- **Private asset type:** Upload as `type: private` with Strict Transformations enabled so raw originals have no public URL.
- **IDOR & CSRF:** Strict ownership verification on upload, replacement, and deletion. Anti-CSRF on every upload form.
- **Delete on replace/removal:** Delete the Cloudinary asset when an image is removed or replaced to prevent storage leaks.

## 4. Visibility (Offers Without an Image)
- An offer without an image is not public: it is excluded from public feeds (`/offers`) and its public detail route (`/offers/{slug}`) returns 404.
- The owner path (`/profile/offers/{id}`) still renders it with an explicit notice: *"Only you can see this offer. Add a photo to make it public."*
- Enforce via database query conditions (stored image count/flag in `haven-db`), never via $O(N)$ application filtering.
- Outages: visibility follows database state, not Cloudinary availability. A Cloudinary outage triggers the failed-image state, never hiding the offer.

## 5. Profile Avatars
- Profile avatars are optional. Unset avatars render the neutral placeholder (initial on gray circle).
- If authenticated via Google or email, avatar upload remains manual opt-in. A Google photo is never imported automatically without explicit choice.
- If an import action is added, haven-web fetches the Google image server-side, validates magic bytes, strips metadata, and uploads to Cloudinary. Never hotlink third-party CDNs.

## 6. Image States & Accessibility
- **Alt text:** Offer title serves as alt text. In cards/rows where title is adjacent text, use `alt=""` to prevent screen reader duplication. On detail pages, use `"Title, photo N of M"`. Profile avatars use the display name.
- **States:**
  - *No image / loading / failed:* Neutral gray placeholder box matching exact dimensions to eliminate layout shift.
  - *At maximum (5 of 5):* Replace upload control with plain text: *"5 of 5 photos. Remove one to add another."*
  - *Last photo protection:* Remove button hidden with explanation: *"An offer needs at least one photo. Add another first, or replace this one."*
  - *Failed upload:* Preserve typed form text; explain plainly that files must be selected again.
