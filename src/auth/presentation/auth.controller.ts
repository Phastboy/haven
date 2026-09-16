import { Elysia } from 'elysia';
import { RequestMagicLinkDTO, VerifyMagicLinkDTO, LoginWithGoogleDTO } from './dtos/auth.dtos';
import { authErrorPlugin } from './auth.error';
import { authMiddleware } from './middleware/session.middleware';

import { RequestMagicLinkUseCase } from '../application/use-cases/request-magic-link.use-case';
import { VerifyMagicLinkUseCase } from '../application/use-cases/verify-magic-link.use-case';
import { LoginWithGoogleUseCase } from '../application/use-cases/login-with-google.use-case';
import { LogoutUseCase } from '../application/use-cases/logout.use-case';
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
import { IEmailService } from '../domain/ports/IEmailService';

import { CreateUserUseCase } from '../../user/application/create-user.usecase';
import { PrismaUserRepository } from '../../user/infrastructure/prisma-user.repository';

const env = process.env['NODE_ENV'] || 'development';
const emailService: IEmailService = 
  (env === 'production' || process.env['SMTP_HOST']) 
    ? new SmtpEmailService() 
    : new ConsoleEmailService();

const magicLinkRepo = new MagicLinkRepository();
const accountRepo = new AccountRepository();
const sessionRepo = new SessionRepository();
const oauthRepo = new OAuthCredentialRepository();
const platformLinkRepo = new AccountPlatformLinkRepository();
const googleService = new GoogleTokenService();

const requestMagicLinkUC = new RequestMagicLinkUseCase(magicLinkRepo, emailService, tokenService);
const verifyMagicLinkUC = new VerifyMagicLinkUseCase(magicLinkRepo, accountRepo, sessionRepo, tokenService);
const loginWithGoogleUC = new LoginWithGoogleUseCase(accountRepo, oauthRepo, sessionRepo, googleService, tokenService);
const logoutUC = new LogoutUseCase(sessionRepo, tokenService);
const linkPlatformUC = new LinkPlatformUseCase(platformLinkRepo);

const createUserUC = new CreateUserUseCase(new PrismaUserRepository());

export const authController = new Elysia({ prefix: '/auth', name: 'auth-controller' })
  .use(authErrorPlugin)
  .use(authMiddleware)
  
  .post('/magic-link/request', {
    body: RequestMagicLinkDTO
  }, async ({ body }) => {
    await requestMagicLinkUC.execute(body.email);
    return { message: 'If the email exists, a magic link was sent.' };
  })

  .post('/magic-link/verify', {
    body: VerifyMagicLinkDTO
  }, async ({ body, request }) => {
    const userAgent = request.headers.get('user-agent') || undefined;
    const ipAddress = request.headers.get('x-forwarded-for') || undefined;
    const session = await verifyMagicLinkUC.execute(body.token, userAgent, ipAddress);

    try {
      const account = await accountRepo.findById(session.accountId);
      if (account) {
        const user = await createUserUC.execute({ email: account.email, username: account.email.split('@')[0] ?? null });
        await linkPlatformUC.execute(account.id, user.id, 'haven_platform');
      }
    } catch (e) {
      console.log('Error in user/link creation (Magic Link):', e);
      // Ignored if user already exists
    }

    return session;
  })

  .post('/google/login', {
    body: LoginWithGoogleDTO
  }, async ({ body, request }) => {
    const userAgent = request.headers.get('user-agent') || undefined;
    const ipAddress = request.headers.get('x-forwarded-for') || undefined;
    const session = await loginWithGoogleUC.execute(body.idToken, userAgent, ipAddress);

    try {
      const account = await accountRepo.findById(session.accountId);
      if (account) {
        const user = await createUserUC.execute({ 
          email: account.email, 
          username: account.email.split('@')[0] ?? null,
        });
        await linkPlatformUC.execute(account.id, user.id, 'haven_platform');
      }
    } catch (e) {
      console.log('Error in user/link creation (Google):', e);
      // Ignored if user already exists
    }

    return session;
  })

  .post('/logout', async ({ headers }) => {
    const authHeader = headers['authorization'];
    if (authHeader && authHeader.startsWith('Bearer ')) {
      const token = authHeader.substring(7);
      await logoutUC.execute(token);
    }
    return { success: true };
  })

  .get('/me', {
    requireAuth: true
  }, ({ account }: any) => {
    return { account };
  });
