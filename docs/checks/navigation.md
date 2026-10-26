# Navigation shell manual check

Run these checks as a signed-in person and as a visitor, on a main page and a child page. Visit the feed, an offer, a public profile, landing, and sign-in. Check each listed viewport width in browser DevTools.

| Width | Expected navigation for main and child pages |
|---:|---|
| 390px | Main: bottom tabs. Child: 56px back-only top bar. |
| 640px | Main: bottom tabs. Child: 56px back-only top bar. |
| 641px | Collapsed 72px rail. Child also has the desktop Back link. |
| 768px | Collapsed 72px rail. Child also has the desktop Back link. |
| 820px | Collapsed 72px rail. Child also has the desktop Back link. |
| 1024px | Collapsed 72px rail. Child also has the desktop Back link. |
| 1180px | Collapsed 72px rail. Child also has the desktop Back link. |
| 1279px | Collapsed 72px rail. Child also has the desktop Back link. |
| 1280px | Full 240px rail. Child also has the desktop Back link. |

For signed-in people, confirm Offers, Messages, and Profile are present in either rail; Messages shows the unread dot when applicable. For visitors, confirm the rail contains Offers and Sign in only. At 641px through 1279px verify H, icon links, tooltips and accessible names; the visitor Sign in control is a person icon. At 1280px and above verify the full wordmark and labels. For auth pages, confirm only Haven and inactive Offers appear, with no Sign in action.

At mobile widths, confirm there is no top navigation on main pages and no tabs on child pages. Confirm safe Back navigation uses a valid same-site referer or falls back to the configured parent; sign-in falls back to `/`.

Browser verification: **Not browser-verified** in this update.
