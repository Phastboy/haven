import type { APIContext } from "astro";
import { client } from "../api";

export type Session = {
  token: string;
  account: { id: string; email: string };
  user: Record<string, any> | null;
};

/**
 * Reads the httpOnly cookie and loads the account + profile in ONE round trip
 * pair instead of two sequential ones. Returns null when signed out.
 */
export async function getSession(ctx: APIContext): Promise<Session | null> {
  const token = ctx.cookies.get("token")?.value;
  if (!token) return null;

  const headers = { authorization: `Bearer ${token}` };
  const [authRes, userRes] = await Promise.all([
    client.auth.me.get({ headers }),
    client.users.me.get({ headers }),
  ]);

  const account = authRes.data?.account;
  if (authRes.error || !account) return null;

  return { token, account, user: (userRes.data as any) ?? null };
}

/**
 * For protected pages. Returns either the session or the Response to return:
 *
 *   const guard = await requireSession(Astro);
 *   if (guard.redirect) return guard.redirect;
 *
 * (A page has to RETURN a redirect — throwing one surfaces as a 500.)
 */
export async function requireSession(
  ctx: APIContext,
): Promise<{ session: Session; redirect: null } | { session: null; redirect: Response }> {
  const session = await getSession(ctx);
  if (!session) {
    return {
      session: null,
      redirect: ctx.redirect(`/login?next=${encodeURIComponent(ctx.url.pathname)}`),
    };
  }
  return { session, redirect: null };
}

export const authHeaders = (token: string) => ({ authorization: `Bearer ${token}` });
