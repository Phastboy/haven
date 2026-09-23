import { treaty } from "@elysia/eden";
import type { App } from "@haven/api";

export const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3000/api";
export const GRAPHQL_URL = `${API_URL}/graphql`;
export const client = treaty<App>(API_URL);
