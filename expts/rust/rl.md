available in v0.2 only means offers made on the platform, so basically only slug is the permitted change that is allowed to happen to offer model.

- **Slug route:** everyone gets the same public page, owners included. The handler doesn't look at who is asking.
- **ID route:** the owner path. The person must be signed in, and the offer must be theirs, checked in the SQL (`WHERE id = $1 AND user_id = $2`).

Implementation details:
1. **The public page never reads the session.** Don't set cookies on it, and don't show owner-only controls there, such as an "Edit" link. Anything that varies by viewer makes the page uncacheable and risks serving one person's version to another. The owner reaches edit and delete from their own offers list, and the public page can ignore them.
2. **Cache headers differ by route.** The public page gets a shared cache policy with an `ETag`. The ID route gets `Cache-Control: private, no-store` and `noindex`, so owner pages never land in a shared cache or a search index.
3. **ID route failures.** Someone else's offer and a missing offer should return the same 404, so ids can't be probed. A signed-out visitor is redirected to sign in.
4. **Canonical URL.** The public page declares its canonical slug URL, and the owner page doesn't compete with it for indexing.

The same offer is reachable through two URLs, so give the ID route its own prefix (for example `/my/offers/{id}`) and keep the slug route under its own. That way one path never has to guess whether it was given a UUID or a slug.

There are no profiles yet, so the public page show absolutely nothing about the maker.

Yes, `/offers/manage/{id}` works, and I'd keep it. Everything owner-only sits under one prefix (`/offers/manage`, `/offers/manage/new`, `/offers/manage/{id}`, `/offers/manage/{id}/edit`, and delete). Your public slug route is `/offers/{slug}`, which has one segment where the owner routes have two or more, so they don't overlap.

Three things keep it safe:
1. **Reserve `manage` and `new`.** A slug should always end in its id suffix, so a bare `/offers/manage` can never be a valid slug. Also check how Topcoat resolves a static segment against a parameter segment, using `rust-crate-docs`, and don't assume it.
2. **Make the prefix carry the rules.** Add `Disallow: /offers/manage/` to `robots.txt`, `noindex` on those pages, and `Cache-Control: private, no-store` on them. `/my/offers/...` would make this a little simpler, since the private area is its own root. Yours is fine as long as the rules are written against the prefix.
3. **Apply the rules without middleware.** `ENGINEERING.md` avoids middleware for auth, so I'd use one shared helper that every owner handler calls for authentication and the private headers.

Add a test that requests every route under `/offers/manage` with no session and expects a refusal. When someone adds a route there later and forgets the auth check, that test fails.
