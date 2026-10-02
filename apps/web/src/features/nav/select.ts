import { items } from "./items";
import type { NavItem, NavSurface } from "./types";

export function navFor(surface: NavSurface, isMember: boolean): NavItem[] {
  return items
    .filter((i) => i.surfaces.includes(surface))
    .filter((i) => i.audience === "both" || isMember)
    .sort((a, b) => a.order - b.order);
}

export function isActive(item: NavItem, path: string): boolean {
  const roots = item.match ?? [item.href];
  return roots.some((root) => path === root || path.startsWith(`${root}/`));
}
