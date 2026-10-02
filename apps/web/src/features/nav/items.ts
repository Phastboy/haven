import type { NavItem } from "./types";

/**
 * The ONLY place a destination is registered. No component lists destinations itself.
 *
 * A top-level destination must be: (1) something a typical user opens weekly,
 * (2) a distinct job, not a view of another destination, (3) not an account utility.
 * iOS and Material cap a tab bar at 5. At 5 now, so a new one must displace one.
 * First candidate to fold into Profile: "My offers" (it is the owner's view of Profile's Offers).
 */
export const items: NavItem[] = [
  {
    id: "directory",
    href: "/directory",
    label: "Directory",
    icon: "M11 3a8 8 0 1 0 4.9 14.32L21 22l1-1-4.68-5.1A8 8 0 0 0 11 3Z",
    audience: "both",
    surfaces: ["desktop", "tabs"],
    order: 0,
  },
  {
    id: "orders",
    href: "/orders",
    label: "Orders",
    icon: "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2 M9 2h6a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1Z M12 11h4 M12 16h4 M8 11h.01 M8 16h.01",
    audience: "member",
    surfaces: ["desktop", "tabs"],
    order: 10,
  },
  {
    id: "messages",
    href: "/messages",
    label: "Messages",
    icon: "M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z M22 6l-10 7L2 6",
    audience: "member",
    surfaces: ["desktop", "tabs"],
    order: 20,
  },
  {
    id: "my-offers",
    href: "/my-offers",
    label: "My offers",
    icon: "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z",
    audience: "member",
    surfaces: ["desktop", "tabs"],
    order: 30,
  },
  {
    id: "profile",
    href: "/profile",
    label: "Profile",
    icon: "M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2M12 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z",
    avatar: true,
    match: ["/profile", "/settings"],
    audience: "member",
    surfaces: ["tabs"],
    order: 40,
  },
];
