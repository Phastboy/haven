# Haven Frontend Plan

## The Golden Rule
The one rule that matters more than any other: **A frontend stage doesn't start until its matching backend stage is already demoed and working.** We do not build screens against pretend data and hope the API matches later — each screen gets built against a real, running feature.

The frontend follows the exact same six stages as the backend, one step behind:
- Backend Stage 1 done → Frontend Stage 1 built
- Backend Stage 2 done → Frontend Stage 2 built
- ... and so on.

## Architecture Stack
- **Framework**: Astro (Server-Side Rendering via Bun)
- **UI Framework**: Solid (Islands Architecture)
- **Styling**: TailwindCSS v4
- **Components**: i don't know if there's anyone or we just have to go raw
- **API Client**: Elysia Eden (`treaty`)

---

## The 6 Stages (Overview)

1. **Stage 1: Auth & App Shell (CURRENT)**
   - Magic Link & Google Authentication
   - Session persistence (Cookies)
   - Global navigation and layout shell
2. **Stage 2: Core Directory & Profile**
   - User onboarding, profile management, and viewing the community directory
3. **Stage 3: Offers & Requests**
   - Creating, editing, and listing offers/requests
4. **Stage 4: Matches**
   - UI for viewing, accepting, and declining matches
5. **Stage 5: Messaging**
   - Live chat UI for matched users
6. **Stage 6: Real-time & Notifications**
   - WebSocket integration for live updates and toasts

---

## Stage 1: Granular Implementation (Astro + Solid)

We use a "True Zero" iterative approach, verifying the build at every micro-step.

- [] **Step 1: Tailwind Integration**
  - Install `@tailwindcss/vite` and configure `astro.config.mjs`.
  - Set up `Layout.astro` and `global.css`.
- [] **Step 2: API Connection (Eden)**
  - Establish the `treaty<App>` connection in `src/lib/api.ts`.
  - Link `@haven/api` workspace to resolve backend types natively.
- [] **Step 3: Minimal Auth Shell (Server-Side SSR)**
  - **Auth Middleware**: In `src/layouts/Layout.astro`, read `Astro.cookies.get("token")`.
  - **Fetch Profile**: If token exists, securely call `api.auth.me.get()` on the server.
  - **Navbar**: Conditionally render "Login" or "Profile" based on the server-rendered auth state. Zero client-side JS required for auth checks.
- [] **Step 4: Magic Link Login (React Island)**
  - **Login Form (`LoginForm.tsx`)**:  that hits `api.auth.magicLink.request.post()`.
  - **Login Page (`login.astro`)**: Hosts the interactive React Island.
  - **Verify Page (`verify.astro`)**: Server-only route that grabs the token from the URL (`?token=XYZ`), validates it if needed, sets the secure cookie via `Astro.cookies.set()`, and instantly redirects to `/`.

---

## Notes from the frontend pass (Stage 1–4 review)

Fixed:
- Session token no longer passed into islands (`src/pages/api/[...path].ts` proxy).
- Mobile: real bottom tab bar, `initial-scale=1`, safe-area padding, 44px targets.
- `getSession` replaces the per-page `auth.me` → `users.me` waterfall.
- Eden types restored (see README: cron plugin annotation + CORS plugin).

Not done yet:
- Stage 5 (messaging) and Stage 6 (realtime) — no WebSocket work at all.
- Directory still uses raw `fetch` for GraphQL, so those fields are unchecked.
- Image upload (offers take a URL).
