import { config } from "../../config";
import type { DB } from "../../database/db";
import { Elysia } from "elysia";
import { NotFoundError } from "../../shared/errors";
import { GetUserUseCase } from "../application/get-user.usecase";
import { ListUsersUseCase } from "../application/list-users.usecase";
import { UpdateUserUseCase } from "../application/update-user.usecase";
import { SqlUserRepository } from "../infrastructure/sql-user.repository";
import { UpdateUserBody, UserIdParam, UserResponse } from "./user.dto";
import { t } from "elysia";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { UnauthorizedError } from "../../auth/domain/errors";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { TokenService } from "../../auth/infrastructure/services/token.service";
import { PaginatedResponseSchema } from "../../shared/domain/pagination";
const tokenService = new TokenService(config);

/**
 * Factory function that wires the full user feature as an Elysia plugin.
 * Dependency injection happens here — the presentation layer owns wiring, not the app root.
 *
 * Elysia 2 route signature: (path, hook, handler) — hook holds schemas/details,
 * handler is the actual function. They are separate positional arguments.
 */
export function createUserPlugin(db: DB) {
  const repository = new SqlUserRepository(db);

  const getUser = new GetUserUseCase(repository);
  const listUsers = new ListUsersUseCase(repository);
  const updateUser = new UpdateUserUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(db), tokenService);

  return new Elysia({ prefix: "/users", tags: ["Users"] })
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

    .get(
      "/",
      {
        response: {
          200: PaginatedResponseSchema(UserResponse),
        },
        detail: { summary: "List all profiles" },
      },
      async () => {
        const users = await listUsers.execute();
        return users; // PaginatedResponse already has data/meta
      },
    )

    .get(
      "/:id",
      {
        params: UserIdParam,
        response: {
          200: t.Object({ data: UserResponse }),
        },
        detail: { summary: "Get a profile by ID" },
      },
      async ({ params }) => {
        const user = await getUser.execute(params.id);
        return { data: user };
      },
    )

    .patch(
      "/me",
      {
        body: UpdateUserBody,
        response: {
          200: t.Object({ data: UserResponse }),
        },
        beforeHandle: [requireAuth],
        detail: { summary: "Update my profile" },
      },
      async ({ account, body }) => {
        const user = await updateUser.execute(account!.id, body);
        return { data: user };
      },
    )

    .get(
      "/me",
      {
        response: {
          200: t.Object({ data: UserResponse }),
        },
        beforeHandle: [requireAuth],
        detail: { summary: "Get my profile" },
      },
      async ({ account }) => {
        const user = await repository.findByAccountId(account!.id);
        if (!user) throw new NotFoundError("User", account!.id);
        return { data: user };
      },
    );
}
