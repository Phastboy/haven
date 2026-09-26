import { treaty } from "@elysia/eden";
import type { App } from "@haven/api";

export const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3000";

// Eden's type inference puts routes under `.api` because the root Elysia app uses prefix: "/api".
// We strip the "/api" suffix from the URL to avoid double-prefixing ("/api/api/auth"),
// then export the inner `.api` object to match the frontend's expected `client.auth` structure.
const BASE_URL = API_URL.endsWith("/api") ? API_URL.slice(0, -4) : API_URL;
export const GRAPHQL_URL = `${BASE_URL}/api/graphql`;
export const client = treaty<App>(BASE_URL).api;
