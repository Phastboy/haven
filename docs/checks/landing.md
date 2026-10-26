# Landing page manual check

Use a browser with the seeded app and visit `/` as a visitor. Repeat the signed-in request to confirm it redirects to `/offers`.

| Width | Checks |
|---|---|
| 390px | No top bar; hero begins at the top with 24px spacing; buttons stack full width; visitor bottom tabs show Offers and Sign in; recently offered uses two equal-width columns. |
| 1024px | Visitor rail is 72px; hero has 64px top spacing; recently offered uses three equal-width columns and shows six cards. |
| 1440px | Visitor rail is 240px; hero has 80px top spacing; recently offered uses four equal-width columns and shows up to eight cards. |

Across widths, check that card heights align within each row, a 50-character title truncates without horizontal scrolling, every anonymous heart links to `/auth/sign-in`, and the sign-in rail action appears only on non-auth pages. With no public offers, confirm the Recently offered heading and section are absent. Confirm the hero actions link to `/offers` and `/auth/sign-in`.

Browser verification: **Not browser-verified** in this implementation session.
