import type { APIRoute } from "astro";
import { API_URL } from "../../api";

// Same-origin proxy for browser islands.
//
// The session token lives in an httpOnly cookie. Islands call /api/... on this
// origin; this route attaches the Authorization header server-side and forwards
// to Elysia. That keeps the token out of the HTML and out of client JS, which
// is the whole point of httpOnly.
const HOP_BY_HOP = new Set(["host", "connection", "content-length", "cookie"]);

const forward: APIRoute = async ({ params, request, cookies, url }) => {
  const token = cookies.get("token")?.value;
  const target = `${API_URL}/${params.path ?? ""}${url.search}`;

  const headers = new Headers();
  request.headers.forEach((value, key) => {
    if (!HOP_BY_HOP.has(key.toLowerCase())) headers.set(key, value);
  });
  if (token) headers.set("authorization", `Bearer ${token}`);

  const res = await fetch(target, {
    method: request.method,
    headers,
    body:
      request.method === "GET" || request.method === "HEAD"
        ? undefined
        : await request.arrayBuffer(),
  });

  const out = new Headers(res.headers);
  out.delete("content-encoding");
  out.delete("content-length");
  return new Response(res.body, { status: res.status, headers: out });
};

export const GET = forward;
export const POST = forward;
export const PATCH = forward;
export const PUT = forward;
export const DELETE = forward;
