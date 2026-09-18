import { Elysia } from 'elysia';
import { NotFoundError, ConflictError } from '../../shared/errors';
import { CreateUserUseCase } from '../application/create-user.usecase';
import { GetUserUseCase } from '../application/get-user.usecase';
import { ListUsersUseCase } from '../application/list-users.usecase';
import { UpdateUserUseCase } from '../application/update-user.usecase';
import { DeleteUserUseCase } from '../application/delete-user.usecase';
import { SqlUserRepository } from '../infrastructure/sql-user.repository';
import { UpdateUserBody, UserIdParam } from './user.dto';
import { requireAuth } from '../../auth/presentation/middleware/session.middleware';

/**
 * Factory function that wires the full user feature as an Elysia plugin.
 * Dependency injection happens here — the presentation layer owns wiring, not the app root.
 *
 * Elysia 2 route signature: (path, hook, handler) — hook holds schemas/details,
 * handler is the actual function. They are separate positional arguments.
 */
export function createUserPlugin() {
  const repository = new SqlUserRepository();

  const getUser = new GetUserUseCase(repository);
  const listUsers = new ListUsersUseCase(repository);
  const updateUser = new UpdateUserUseCase(repository);

  return new Elysia({ prefix: '/users', tags: ['Users'] })
    .error(ConflictError, ({ set, error }) => {
      set.status = 409;
      return { error: error.message };
    })
    .error(NotFoundError, ({ set, error }) => {
      set.status = 404;
      return { error: error.message };
    })

    .get('/',
      { detail: { summary: 'List all profiles' } },
      async () => listUsers.execute(),
    )

    .get('/:id',
      { params: UserIdParam, detail: { summary: 'Get a profile by ID' } },
      async ({ params }) => getUser.execute(params.id),
    )

    .patch('/me',
      { body: UpdateUserBody, beforeHandle: [requireAuth], detail: { summary: 'Update my profile' } },
      async ({ account, body }) => updateUser.execute(account!.id, body),
    );
}
