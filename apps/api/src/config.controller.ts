import { Elysia } from "elysia";
import { verifyConfigSignature } from "./config/config-auth.middleware";

/**
 * /api/config — returns client-side configuration values.
 *
 * This endpoint is gated by an Ed25519 request-signature check.
 * Only the official web client (which holds CONFIG_PRIVATE_KEY) can call it.
 * See src/config/config-auth.middleware.ts for the full protocol.
 */
export const configController = new Elysia({
  prefix: "/config",
  name: "config-controller",
  tags: ["Config"],
}).get(
  "/",
  {
    detail: { summary: "Get application configuration (requires signed request)" },
  },
  ({ headers, set }) => {
    const authError = verifyConfigSignature({ headers, set });
    if (authError) return authError;

    return {
      googleClientId: process.env["GOOGLE_CLIENT_ID"] || "",
      primeUiLicense: process.env["PRIME_UI_LICENSE"] || "",
    };
  },
);
