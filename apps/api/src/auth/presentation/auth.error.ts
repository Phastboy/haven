import Elysia from 'elysia';
import { AuthError, UnauthorizedError, InvalidTokenError, ExpiredTokenError, AccountNotFoundError } from '../domain/errors';

export const authErrorPlugin = new Elysia({ name: 'auth-error-handler' })
  .error(({ error, set }) => {
    if (error instanceof InvalidTokenError || error instanceof ExpiredTokenError || error.name === 'InvalidTokenError') {
      set.status = 400;
      return { message: error.message };
    }
    if (error instanceof UnauthorizedError || error.name === 'UnauthorizedError') {
      set.status = 401;
      return { message: error.message };
    }
    if (error instanceof AccountNotFoundError || error.name === 'AccountNotFoundError') {
      set.status = 404;
      return { message: error.message };
    }
    if (error instanceof AuthError || error.name === 'AuthError') {
      set.status = 400;
      return { message: error.message };
    }
    return;
  });
