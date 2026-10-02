import type { AstroComponentFactory } from "astro/runtime/server/index.js";

export type NavSurface = "desktop" | "tabs";

export type NavItem = {
  id: string;
  href: string;
  label: string;
  /** SVG path data (24x24, stroke icon). */
  icon: string;
  /** Show the user's avatar instead of `icon` (the Profile tab, as Instagram does). */
  avatar?: boolean;
  /** Paths that mark this item active. Defaults to [href]. */
  match?: string[];
  audience: "member" | "both";
  /** Where the item appears. Profile is tabs-only: on desktop it lives in the avatar menu. */
  surfaces: NavSurface[];
  /** Lower first. Leave gaps so inserting one never renumbers the others. */
  order: number;
  /**
   * Optional count/dot owned by the item's own feature (e.g. unread messages).
   * Not wired yet: it needs a client-side island so the layout never waits on an API call.
   */
  badge?: AstroComponentFactory;
};

/** What the navigation is allowed to know about the signed-in user. */
export type NavUser = {
  email: string;
  name: string | null;
  username: string | null;
  profilePictureUrl: string | null;
};
