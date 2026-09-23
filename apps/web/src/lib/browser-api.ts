import { treaty } from "@elysia/eden";
import type { App } from "@haven/api";

/**
 * Eden client for Solid islands. It points at THIS origin's /api proxy, which
 * adds the session token from the httpOnly cookie. Islands never touch the token.
 */
export const api = treaty<App>(
  import.meta.env.SSR ? "http://localhost:4200/api" : `${window.location.origin}/api`,
);
