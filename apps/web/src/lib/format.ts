// Explicit locale everywhere so the server and the browser render identical
// text (a mismatch would show up as a hydration warning).
const money = new Intl.NumberFormat("en-US", { style: "currency", currency: "USD" });
const date = new Intl.DateTimeFormat("en-US", { month: "short", day: "numeric", year: "numeric" });

/** Prices are stored in cents. */
export const formatPrice = (cents: number | null | undefined) =>
  typeof cents === "number" && Number.isFinite(cents) ? money.format(cents / 100) : "—";

export const formatDate = (value: string | Date | null | undefined) =>
  value ? date.format(new Date(value)) : "";

export const initials = (name?: string | null, username?: string | null) =>
  (name || username || "?").trim().charAt(0).toUpperCase();
