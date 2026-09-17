import Elysia from 'elysia';
import { AuthError, UnauthorizedError, InvalidTokenError, ExpiredTokenError, AccountNotFoundError } from '../domain/errors';

export const authErrorPlugin = new Elysia({ name: 'auth-error-handler' })
  .error(AuthError, ({ set, error }) => {
    set.status = 400;
    return { message: error.message };
  })
  .error(UnauthorizedError, ({ set, error }) => {
    set.status = 401;
    return { message: error.message };
  })
  .error(InvalidTokenError, ({ set, error }) => {
    set.status = 400;
    return { message: error.message };
  })
  .error(ExpiredTokenError, ({ set, error }) => {
    set.status = 400;
    return { message: error.message };
  })
  .error(AccountNotFoundError, ({ set, error }) => {
    set.status = 404;
    return { message: error.message };
  });
