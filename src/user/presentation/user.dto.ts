import { t } from 'elysia';

/**
 * TypeBox schemas used by Elysia for both runtime validation and OpenAPI generation.
 */

export const CreateUserBody = t.Object({
  email: t.String({ format: 'email', description: 'User email address' }),
  username: t.Optional(t.String({ minLength: 2, description: 'Optional username' })),
  name: t.Optional(t.String({ minLength: 1, description: 'Optional display name' })),
});

export const UpdateUserBody = t.Object({
  username: t.Optional(t.Union([t.String({ minLength: 2 }), t.Null()])),
  name: t.Optional(t.Union([t.String({ minLength: 1 }), t.Null()])),
});

export const UserIdParam = t.Object({
  id: t.String({ description: 'User UUID' }),
});

export const UserResponse = t.Object({
  id: t.String(),
  email: t.String(),
  username: t.Union([t.String(), t.Null()]),
  name: t.Union([t.String(), t.Null()]),
  createdAt: t.String(),
  updatedAt: t.String(),
});
