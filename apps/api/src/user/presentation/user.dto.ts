import { t } from 'elysia';
import {
  UserSchema,
  CreateUserDataSchema,
  UpdateUserDataSchema,
} from '../domain/user.schema';

/**
 * HTTP-layer TypeBox schemas.
 *
 * These are built on top of the domain schemas, adding presentation-layer
 * concerns: format annotations (for OpenAPI), minLength constraints (for early
 * rejection before use cases run), and field descriptions.
 *
 * `UserResponse` is the domain UserSchema re-used directly — the HTTP response
 * shape and the domain entity shape are intentionally the same at this stage.
 */


export const UpdateUserBody = t.Object({
  username: t.Optional(t.Union([t.String({ minLength: 2 }), t.Null()])),
  name: t.Optional(t.Union([t.String({ minLength: 1 }), t.Null()])),
  bio: t.Optional(t.Union([t.String({ maxLength: 500 }), t.Null()])),
  profilePictureUrl: t.Optional(t.Union([t.String({ format: 'uri' }), t.Null()])),
});

export const UserIdParam = t.Object({
  id: t.String({ description: 'User UUID' }),
});

/**
 * The HTTP response schema is the domain schema — they are the same shape.
 * If the HTTP response ever needs to differ from the domain entity (e.g. omitting
 * a field or adding computed values), split them here and update the plugin mapper.
 */
export const UserResponse = UserSchema;

// Re-export domain schemas for use-case input validation if needed downstream.
export { CreateUserDataSchema, UpdateUserDataSchema };
