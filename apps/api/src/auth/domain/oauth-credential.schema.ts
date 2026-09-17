import { Type, Static } from 'typebox';

export const OAuthCredentialSchema = Type.Object({
  id: Type.String({ format: 'uuid' }),
  accountId: Type.String({ format: 'uuid' }),
  provider: Type.Union([Type.Literal('GOOGLE')]),
  providerUserId: Type.String(),
  accessToken: Type.String(),
  refreshToken: Type.Optional(Type.String()),
  tokenExpiresAt: Type.Optional(Type.String({ format: 'date-time' })),
  createdAt: Type.String({ format: 'date-time' }),
  updatedAt: Type.String({ format: 'date-time' }),
});

export type OAuthCredential = Static<typeof OAuthCredentialSchema>;
