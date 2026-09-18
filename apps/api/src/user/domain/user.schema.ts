import { t } from 'elysia';
import type { Static } from 'typebox';

/**
 * Domain-level TypeBox schemas.
 *
 * These are the single source of truth for the User domain shapes.
 * TypeScript types are DERIVED from these schemas — no manual interface
 * definitions that can silently drift from the runtime schema.
 *
 * Types are extracted via `(typeof schema)['static']`, which is the canonical
 * TypeBox pattern that works across any TypeBox fork (including Elysia's bundled
 * version) without needing to import a `Static` helper from a specific package.
 *
 * No HTTP-specific constraints here (no format, minLength, descriptions).
 * Those belong to the presentation layer where they annotate the HTTP contract.
 */

export const UserSchema = t.Object({
  id: t.String(),
  accountId: t.String(),
  username: t.Union([t.String(), t.Null()]),
  name: t.Union([t.String(), t.Null()]),
  bio: t.Union([t.String(), t.Null()]),
  profilePictureUrl: t.Union([t.String(), t.Null()]),
  createdAt: t.String(),
  updatedAt: t.String(),
});

export const CreateUserDataSchema = t.Object({
  accountId: t.String(),
  username: t.Optional(t.Union([t.String(), t.Null()])),
  name: t.Optional(t.Union([t.String(), t.Null()])),
  bio: t.Optional(t.Union([t.String(), t.Null()])),
  profilePictureUrl: t.Optional(t.Union([t.String(), t.Null()])),
});

export const UpdateUserDataSchema = t.Object({
  username: t.Optional(t.Union([t.String(), t.Null()])),
  name: t.Optional(t.Union([t.String(), t.Null()])),
  bio: t.Optional(t.Union([t.String(), t.Null()])),
  profilePictureUrl: t.Optional(t.Union([t.String(), t.Null()])),
});

/** The full user record as the domain works with it. */
export type User = Static<typeof UserSchema>;

/** Input required to create a user. */
export type CreateUserData = Static<typeof CreateUserDataSchema>;

/** Fields a caller may update on a user. */
export type UpdateUserData = Static<typeof UpdateUserDataSchema>;
