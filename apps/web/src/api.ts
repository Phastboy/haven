import { treaty } from "@elysia/eden";
import type { App } from "@haven/api";

const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3000/api";
export const client = treaty<App>(API_URL);
