import { Elysia, t } from "elysia";
import {
  RequestMagicLinkDTO,
  VerifyMagicLinkDTO,
  LoginWithGoogleDTO,
  RequestMagicLinkResponseDTO,
  AuthSuccessResponseDTO,
  MeResponseDTO,
  ErrorResponseDTO,
} from "./dtos/auth.dtos";
import { requireAuth } from "./middleware/session.middleware";
import { UnauthorizedError } from "../domain/errors";

import { RequestMagicLinkUseCase } from "../application/use-cases/request-magic-link.use-case";
import { VerifyMagicLinkUseCase } from "../application/use-cases/verify-magic-link.use-case";
import { LoginWithGoogleUseCase } from "../application/use-cases/login-with-google.use-case";
import { LogoutUseCase } from "../application/use-cases/logout.use-case";
import { GetSessionUseCase } from "../application/use-cases/get-session.use-case";

import { MagicLinkRepository } from "../infrastructure/repositories/magic-link.repository";
import { AccountRepository } from "../infrastructure/repositories/account.repository";
import { SessionRepository } from "../infrastructure/repositories/session.repository";
import { OAuthCredentialRepository } from "../infrastructure/repositories/oauth-credential.repository";
import type { IProfileCreator } from "../domain/ports/IProfileCreator";

import { tokenService } from "../infrastructure/services/token.service";
import { GoogleTokenService } from "../infrastructure/services/google-token.service";
import { SmtpEmailService } from "../infrastructure/services/smtp-email.service";
import { ConsoleEmailService } from "../infrastructure/services/console-email.service";

export const createAuthPlugin = (profileCreator: IProfileCreator) => {
  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(), tokenService);

  // Instantiate repositories
  const magicLinkRepo = new MagicLinkRepository();
  const accountRepo = new AccountRepository();
  const sessionRepo = new SessionRepository();
  const oauthRepo = new OAuthCredentialRepository();

  // Determine which email service to use based on environment
  const emailService = process.env["SMTP_HOST"]
    ? new SmtpEmailService()
    : new ConsoleEmailService();

  const googleTokenService = new GoogleTokenService();

  // Instantiate use cases
  const requestMagicLinkUC = new RequestMagicLinkUseCase(magicLinkRepo, emailService, tokenService);
  const verifyMagicLinkUC = new VerifyMagicLinkUseCase(
    magicLinkRepo,
    accountRepo,
    sessionRepo,
    tokenService,
    profileCreator,
  );
  const loginWithGoogleUC = new LoginWithGoogleUseCase(
    accountRepo,
    oauthRepo,
    sessionRepo,
    googleTokenService,
    tokenService,
    profileCreator,
  );
  // Link platform use case is wired up but not yet exposed in any route
  // const linkPlatformUC = new LinkPlatformUseCase(linkRepo);
  const logoutUC = new LogoutUseCase(sessionRepo, tokenService);

  return new Elysia({
    prefix: "/auth",
    name: "auth-plugin",
    tags: ["Auth"],
  })
    .derive(async ({ headers }: { headers: Record<string, string | undefined> }) => {
      const authHeader = headers["authorization"];
      if (!authHeader || !authHeader.startsWith("Bearer ")) {
        return { session: null, account: null };
      }

      const token = authHeader.substring(7);
      try {
        const sessionWithAccount = await getSessionUseCase.execute(token);
        return {
          session: sessionWithAccount,
          account: sessionWithAccount.account,
        };
      } catch (e: unknown) {
        if (e instanceof UnauthorizedError) {
          return { session: null, account: null };
        }
        throw e;
      }
    })
    .post(
      "/magic-link/request",
      {
        body: RequestMagicLinkDTO,
        response: {
          200: RequestMagicLinkResponseDTO,
          400: ErrorResponseDTO,
          422: ErrorResponseDTO,
          500: ErrorResponseDTO,
        },
      },
      async ({ body }) => {
        await requestMagicLinkUC.execute(body.email);
        return { data: { message: "If the email exists, a magic link was sent to it." } };
      },
    )

    .post(
      "/magic-link/verify",
      {
        body: VerifyMagicLinkDTO,
        response: {
          200: AuthSuccessResponseDTO,
          401: ErrorResponseDTO,
          404: ErrorResponseDTO,
          422: ErrorResponseDTO,
          500: ErrorResponseDTO,
        },
      },
      async ({ body }) => {
        const result = await verifyMagicLinkUC.execute(body.token);
        return { data: result };
      },
    )

    .post(
      "/google/login",
      {
        body: LoginWithGoogleDTO,
        response: {
          200: AuthSuccessResponseDTO,
          401: ErrorResponseDTO,
          422: ErrorResponseDTO,
          500: ErrorResponseDTO,
        },
      },
      async ({ body }) => {
        const result = await loginWithGoogleUC.execute(body.idToken);
        return { data: result };
      },
    )

    .post(
      "/logout",
      {
        beforeHandle: [requireAuth],
        response: {
          204: t.Undefined(),
          401: ErrorResponseDTO,
          500: ErrorResponseDTO,
        },
      },
      async ({
        headers,
        set,
      }: {
        headers: Record<string, string | undefined>;
        set: { status?: number | string };
      }) => {
        const authHeader = headers["authorization"];
        if (authHeader) {
          const token = authHeader.substring(7);
          await logoutUC.execute(token);
        }
        set.status = 204;
        return;
      },
    )

    .get(
      "/me",
      {
        beforeHandle: [requireAuth],
        response: {
          200: MeResponseDTO,
          401: ErrorResponseDTO,
          500: ErrorResponseDTO,
        },
      },
      ({ account }) => {
        return { data: { account: account! } };
      },
    );
};
