import type { Session } from "../../lib/session";
import type { NavUser } from "./types";

/** The one place the session is narrowed to what navigation needs. No `as any` in the layout. */
export function toNavUser(session: Session | null): NavUser | null {
  if (!session) return null;
  const profile: Record<string, unknown> = session.user ?? {};
  return {
    email: session.account.email,
    name: (profile.name as string | null | undefined) ?? null,
    username: (profile.username as string | null | undefined) ?? null,
    profilePictureUrl: (profile.profilePictureUrl as string | null | undefined) ?? null,
  };
}
