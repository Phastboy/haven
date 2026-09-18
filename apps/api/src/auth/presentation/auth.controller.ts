import { Elysia } from 'elysia';
import { RequestMagicLinkDTO, VerifyMagicLinkDTO, LoginWithGoogleDTO } from './dtos/auth.dtos';
import { authErrorPlugin } from './auth.error';
import { requireAuth } from './middleware/session.middleware';
import { UnauthorizedError } from '../domain/errors';

import { RequestMagicLinkUseCase } from '../application/use-cases/request-magic-link.use-case';
import { VerifyMagicLinkUseCase } from '../application/use-cases/verify-magic-link.use-case';
import { LoginWithGoogleUseCase } from '../application/use-cases/login-with-google.use-case';
import { LogoutUseCase } from '../application/use-cases/logout.use-case';
import { GetSessionUseCase } from '../application/use-cases/get-session.use-case';
import { LinkPlatformUseCase } from '../application/use-cases/link-platform.use-case';

import { MagicLinkRepository } from '../infrastructure/repositories/magic-link.repository';
import { AccountRepository } from '../infrastructure/repositories/account.repository';
import { SessionRepository } from '../infrastructure/repositories/session.repository';
import { OAuthCredentialRepository } from '../infrastructure/repositories/oauth-credential.repository';
import { AccountPlatformLinkRepository } from '../infrastructure/repositories/account-platform-link.repository';

import { tokenService } from '../infrastructure/services/token.service';
import { GoogleTokenService } from '../infrastructure/services/google-token.service';
import { SmtpEmailService } from '../infrastructure/services/smtp-email.service';
import { ConsoleEmailService } from '../infrastructure/services/console-email.service';

const getSessionUseCase = new GetSessionUseCase(
  new SessionRepository(),
  tokenService
);

// Instantiate repositories
const magicLinkRepo = new MagicLinkRepository();
const accountRepo = new AccountRepository();
const sessionRepo = new SessionRepository();
const oauthRepo = new OAuthCredentialRepository();
const linkRepo = new AccountPlatformLinkRepository();

// Determine which email service to use based on environment
const emailService = process.env['SMTP_HOST']
  ? new SmtpEmailService()
  : new ConsoleEmailService();

const googleTokenService = new GoogleTokenService();

// Instantiate use cases
const requestMagicLinkUC = new RequestMagicLinkUseCase(magicLinkRepo, emailService, tokenService);
const verifyMagicLinkUC = new VerifyMagicLinkUseCase(magicLinkRepo, accountRepo, sessionRepo, tokenService);
const loginWithGoogleUC = new LoginWithGoogleUseCase(accountRepo, oauthRepo, sessionRepo, googleTokenService, tokenService);
const linkPlatformUC = new LinkPlatformUseCase(linkRepo);
const logoutUC = new LogoutUseCase(sessionRepo, tokenService);

export const authController = new Elysia({ prefix: '/auth', name: 'auth-controller', tags: ['Auth'] })
  .use(authErrorPlugin)
  .derive(async (ctx) => {
    const authHeader = ctx.headers['authorization'];
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
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
      if ((e as Error).name === 'UnauthorizedError' || e instanceof UnauthorizedError) {
        return { session: null, account: null };
      }
      throw e;
    }
  })
  
  .post('/magic-link/request', {
    body: RequestMagicLinkDTO
  }, async ({ body }) => {
    await requestMagicLinkUC.execute(body.email);
    return { message: 'If the email exists, a magic link was sent to it.' };
  })

  .post('/magic-link/verify', {
    body: VerifyMagicLinkDTO
  }, async ({ body }) => {
    const result = await verifyMagicLinkUC.execute(body.token);
    return result;
  })
  
  .post('/google/login', {
    body: LoginWithGoogleDTO
  }, async ({ body }) => {
    const result = await loginWithGoogleUC.execute(body.idToken);
    return result;
  })

  .post('/logout', {
    beforeHandle: [requireAuth]
  }, async ({ headers }) => {
    const authHeader = headers['authorization']!;
    const token = authHeader.substring(7);
    await logoutUC.execute(token);
    return { success: true };
  })

  .get('/me', {
    beforeHandle: [requireAuth]
  }, ({ account }) => {
    return { account };
  });
