import { Elysia } from 'elysia';
import { NotFoundError, ConflictError } from '../../shared/errors';
import { CreateUserUseCase } from '../application/create-user.usecase';
import { GetUserUseCase } from '../application/get-user.usecase';
import { ListUsersUseCase } from '../application/list-users.usecase';
import { UpdateUserUseCase } from '../application/update-user.usecase';
import { DeleteUserUseCase } from '../application/delete-user.usecase';
import { PrismaUserRepository } from '../infrastructure/prisma-user.repository';
import { CreateUserBody, UpdateUserBody, UserIdParam } from './user.dto';

/**
 * Factory function that wires the full user feature as an Elysia plugin.
 * Dependency injection happens here — the presentation layer owns wiring, not the app root.
 *
 * Elysia 2 route signature: (path, hook, handler) — hook holds schemas/details,
 * handler is the actual function. They are separate positional arguments.
 */
export function createUserPlugin() {
  const repository = new PrismaUserRepository();

  const createUser = new CreateUserUseCase(repository);
  const getUser = new GetUserUseCase(repository);
  const listUsers = new ListUsersUseCase(repository);
  const updateUser = new UpdateUserUseCase(repository);
  const deleteUser = new DeleteUserUseCase(repository);

  return new Elysia({ prefix: '/users', tags: ['Users'] })
    .error(ConflictError, ({ set, error }) => {
      set.status = 409;
      return { error: error.message };
    })
    .error(NotFoundError, ({ set, error }) => {
      set.status = 404;
      return { error: error.message };
    })

    .post('/',
      { body: CreateUserBody, detail: { summary: 'Create a new user' } },
      async ({ body, set }) => {
        const user = await createUser.execute(body);
        set.status = 201;
        return user;
      },
    )

    .get('/',
      { detail: { summary: 'List all users' } },
      async () => listUsers.execute(),
    )

    .get('/:id',
      { params: UserIdParam, detail: { summary: 'Get a user by ID' } },
      async ({ params }) => getUser.execute(params.id),
    )

    .patch('/:id',
      { params: UserIdParam, body: UpdateUserBody, detail: { summary: 'Update a user' } },
      async ({ params, body }) => updateUser.execute(params.id, body),
    )

    .delete('/:id',
      { params: UserIdParam, detail: { summary: 'Delete a user' } },
      async ({ params, set }) => {
        await deleteUser.execute(params.id);
        set.status = 204;
      },
    );
}
