import { Type, Static } from 'typebox';
import { AccountSchema } from './account.schema';

export const SessionSchema = Type.Object({
  id: Type.String({ format: 'uuid' }),
  accountId: Type.String({ format: 'uuid' }),
  token: Type.String(),
  expiresAt: Type.String({ format: 'date-time' }),
  userAgent: Type.Optional(Type.String()),
  ipAddress: Type.Optional(Type.String()),
  createdAt: Type.String({ format: 'date-time' }),
});

export type Session = Static<typeof SessionSchema>;

export const SessionWithAccountSchema = Type.Intersect([
  SessionSchema,
  Type.Object({
    account: AccountSchema,
  }),
]);

export type SessionWithAccount = Static<typeof SessionWithAccountSchema>;
