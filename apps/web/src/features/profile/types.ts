import type { AstroComponentFactory } from "astro/runtime/server/index.js";

/**
 * What a profile section is allowed to know about a user.
 * Mirrors the API's UserSchema minus internals (accountId, updatedAt).
 */
export type ProfileUser = {
  id: string;
  username: string;
  name: string | null;
  bio: string | null;
  profilePictureUrl: string | null;
  profileComplete: boolean;
  createdAt: string;
};

/** The only props the assembly hands to a section. Anything else, a section fetches itself. */
export type SectionProps = {
  subject: ProfileUser;
  isOwner: boolean;
};

export type ProfileSection = {
  id: string;
  /** Lower renders first. Leave gaps (0, 10, 20) so inserting one never renumbers the others. */
  order: number;
  audience: "owner" | "visitor" | "both";
  component: AstroComponentFactory;
};
