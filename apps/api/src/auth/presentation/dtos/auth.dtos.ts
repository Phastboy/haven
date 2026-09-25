import { t } from "elysia";
import { AccountSchema } from "../../domain/account.schema";
import { SessionSchema } from "../../domain/session.schema";

// RFC 9457-based error response. Stricter than the bare RFC: every
// base member is REQUIRED by house rule, since RFC 9457's own optionality
// permits a legal-but-useless {} response, which we don't want possible.
// "type" defaults to "about:blank" (never omitted) since we don't
// maintain dereferenceable error-type docs.
export const ErrorResponseDTO = t.Object({
  // Always "about:blank" for now — required so no handler can omit it,
  // not because the RFC demands the field be present.
  type: t.String(),

  // The HTTP reason phrase for `status` ("Not Found", "Unauthorized", ...).
  // Required, and should be derived from `status` by the shared handler,
  // never hand-written per error, so it can't drift from the status code.
  title: t.String(),

  // Duplicates the HTTP status line inside the body, per RFC recommendation.
  // Required so clients parsing only the body still get it.
  status: t.Number(),

  // The one field that's actually meant to vary per occurrence — the
  // specific, human-readable explanation. Required: this is the field
  // most likely to be forgotten/left blank, so it's worth enforcing hardest.
  detail: t.String(),

  // Request path (or path+query, per your earlier decision) identifying
  // this specific occurrence. Required — sourced automatically from the
  // request in the global handler, never supplied by individual routes.
  instance: t.String(),

  // Extension member, RFC-permitted but RFC-undefined. Optional because
  // it's only meaningful for 422 validation failures — enforcing it as
  // required elsewhere would force nonsensical empty arrays on every
  // 401/404/500.
  errors: t.Optional(
    t.Array(
      t.Object({
        name: t.String(),
        reason: t.String(),
      }),
    ),
  ),
});

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
