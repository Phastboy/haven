import { t } from "elysia";
import { AccountSchema } from "../../domain/account.schema";
import { SessionSchema } from "../../domain/session.schema";

export const RequestMagicLinkDTO = t.Object({
  email: t.String({ format: "email" }),
});

export const VerifyMagicLinkDTO = t.Object({
  token: t.String(),
});

export const LoginWithGoogleDTO = t.Object({
  idToken: t.String(),
});

export const RequestMagicLinkResponseDTO = t.Object({
  data: t.Object({
    message: t.String(),
  }),
});

export const AuthSuccessResponseDTO = t.Object({
  data: SessionSchema,
});

export const MeResponseDTO = t.Object({
  data: t.Object({
    account: AccountSchema,
  }),
});
