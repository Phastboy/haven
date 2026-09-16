import { Type, Static } from 'typebox';

export const AccountSchema = Type.Object({
  id: Type.String({ format: 'uuid' }),
  email: Type.String({ format: 'email' }),
  emailVerified: Type.Boolean(),
  createdAt: Type.String({ format: 'date-time' }),
  updatedAt: Type.String({ format: 'date-time' }),
});

export type Account = Static<typeof AccountSchema>;

export const CreateAccountSchema = Type.Omit(AccountSchema, ['id', 'createdAt', 'updatedAt']);
export type CreateAccountDTO = Static<typeof CreateAccountSchema>;
