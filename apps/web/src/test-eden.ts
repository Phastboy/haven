import { treaty } from "@elysia/eden";
import type { App } from "@haven/api";
export const client = treaty<App>("http://localhost:3000");
client.api.auth; // should work if it's api.auth
