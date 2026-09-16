import { Type, Static } from 'typebox';

export const MagicLinkSchema = Type.Object({
  id: Type.String({ format: 'uuid' }),
  email: Type.String({ format: 'email' }),
  token: Type.String(),
  expiresAt: Type.String({ format: 'date-time' }),
  usedAt: Type.Optional(Type.String({ format: 'date-time' })),
  createdAt: Type.String({ format: 'date-time' }),
});

export type MagicLink = Static<typeof MagicLinkSchema>;
