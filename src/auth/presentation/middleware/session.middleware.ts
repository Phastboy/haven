import Elysia from 'elysia';
import { GetSessionUseCase } from '../../application/use-cases/get-session.use-case';
import { SessionRepository } from '../../infrastructure/repositories/session.repository';
import { tokenService } from '../../infrastructure/services/token.service';

const getSessionUseCase = new GetSessionUseCase(
  new SessionRepository(),
  tokenService
);

export const authMiddleware = new Elysia({ name: 'auth-middleware' })
  .derive(async ({ headers }) => {
    // Look for Bearer token in Authorization header
    const authHeader = headers['authorization'];
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
    } catch (e) {
      return { session: null, account: null };
    }
  })
  .macro({
    requireAuth(value: boolean) {
      if (!value) return;
      return {
        beforeHandle({ session, error }: any) {
          if (!session) {
            return error(401, { message: 'Unauthorized access' });
          }
        }
      };
    }
  });
