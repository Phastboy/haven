import { client } from "../../api";
import type { ProfileUser } from "./types";

/**
 * Picks ONLY the public fields. The API currently returns the full domain entity
 * (including accountId) from GET /users/:id, so the web layer must not pass it on.
 * Delete this guard's reason once the API has a public response schema.
 */
export function toProfileUser(raw: Record<string, unknown>): ProfileUser {
  return {
    id: raw.id as string,
    username: raw.username as string,
    name: (raw.name as string | null) ?? null,
    bio: (raw.bio as string | null) ?? null,
    profilePictureUrl: (raw.profilePictureUrl as string | null) ?? null,
    profileComplete: Boolean(raw.profileComplete),
    createdAt: raw.createdAt as string,
  };
}

/**
 * THE identity seam. Today a handle is a user id, because the API only has
 * GET /users/:id. When a username lookup exists, this is the only file that changes.
 */
export async function resolveSubject(handle: string): Promise<ProfileUser | null> {
  const { data, error } = await client.users({ id: handle }).get();
  if (error) {
    if (error.status === 404) return null;
    throw error;
  }
  if (!data) return null;
  const body = (data as { data?: Record<string, unknown> }).data;
  return body ? toProfileUser(body) : null;
}
