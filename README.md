# Haven

A community directory for skills and services: people post offers and requests, and
other members request them.

- `apps/api` — Elysia 2 on Bun. Feature folders (`auth`, `user`, `offer`, `order`,
  `fulfillment`, `directory`), each split into domain / application / infrastructure /
  presentation. Drizzle + Postgres, TypeBox schemas, OpenAPI, GraphQL for the directory.
- `apps/web` — Astro 7 (SSR via Bun) with Solid islands and Tailwind v4.
- Types flow from the API into the web app through Eden Treaty (`treaty<App>`).

## Run

```bash
bun install
docker compose up -d      # Postgres
cp .env.example .env       # and apps/api/.env
bun run --cwd apps/api db:push
bun run dev                # api :3000 + web :4200
```

## Scripts

| Command | What it does |
| --- | --- |
| `bun run dev` | API and web in parallel |
| `bun run typecheck` | `tsc --noEmit` per package |
| `bun run typecheck:web` | `astro check` (templates + islands) |
| `bun run build:web` | Production build of the web app |
| `bun run start:web` | Serve the built web app |
| `bun run test:api` | API tests (pushes the schema first) |

## How the web app is wired

**Auth.** The session token lives in an httpOnly cookie. `src/lib/session.ts` reads it
during SSR and loads the account and profile in parallel:

```ts
const guard = await requireSession(Astro);
if (guard.redirect) return guard.redirect;   // a page must RETURN a redirect;
const session = guard.session;               // throwing one surfaces as a 500
```

**Islands never see the token.** `src/pages/api/[...path].ts` is a same-origin proxy:
browser components call `/api/...` through `src/lib/browser-api.ts`, and the proxy
attaches the `Authorization` header server-side. Passing `token={token}` into an island
would serialise the session into the HTML and hand it to any script on the page, which
defeats the point of httpOnly.

**Islands.** `client:load` only where the component is the page's main interaction
(login form, profile form, the Request button in the sticky bar). Lists use
`client:visible` so cards hydrate when scrolled to. Everything else is plain HTML.

**Mobile first.** Every page is laid out for a 360px screen and widens from there:
a bottom tab bar instead of a hover menu (touch devices have no hover), `min-h-11`
tap targets, `env(safe-area-inset-*)` padding for the notch and home indicator,
16px inputs so iOS doesn't zoom on focus, and `viewport-fit=cover` with
`initial-scale=1`.

## Conventions

- Prices are stored in **cents**. Format with `formatPrice` from `src/lib/format.ts`
  (fixed `en-US` locale, so SSR and the browser produce identical text).
- Pin exact versions of `elysia` and `@elysia/*` in every package. The floating `next`
  tag can resolve to different betas per package, which gives two copies of `elysia` and
  a confusing `Property '#private' ... refers to a different member` error from
  `treaty<App>`. Check with:
  `find . -path '*node_modules/elysia/package.json' -exec grep -H '"version"' {} \;`
- Elysia 2 route signature is `(path, hook, handler)` — the hook comes first.

## Known gaps

- The directory page queries GraphQL with raw `fetch`, so those fields are not checked
  by Eden. Typed by hand in `src/pages/directory.astro` for now.
- Image uploads are not implemented; offers take an image URL.
- No client-side routing: navigation is a full page load. Astro's `<ClientRouter />`
  can change that if the app starts to feel slow between screens.
