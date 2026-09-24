import type { APIRoute } from "astro";
import { client } from "../api";

export const POST: APIRoute = async (ctx) => {
  const token = ctx.cookies.get("token")?.value;
  if (token) {
    try {
      await client.auth.logout.post(null, {
        headers: { authorization: `Bearer ${token}` },
      });
    } catch {
      // Ignore errors on logout
    }
    ctx.cookies.delete("token", { path: "/" });
  }
  return ctx.redirect("/login");
};
