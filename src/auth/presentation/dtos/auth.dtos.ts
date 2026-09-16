import { t } from 'elysia';

export const RequestMagicLinkDTO = t.Object({
  email: t.String({ format: 'email' }),
});

export const VerifyMagicLinkDTO = t.Object({
  token: t.String(),
});

export const LoginWithGoogleDTO = t.Object({
  idToken: t.String(),
});
