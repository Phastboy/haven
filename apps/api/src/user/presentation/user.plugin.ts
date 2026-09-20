import { Elysia } from "elysia";
import { NotFoundError, ConflictError } from "../../shared/errors";
import { GetUserUseCase } from "../application/get-user.usecase";
import { ListUsersUseCase } from "../application/list-users.usecase";
import { UpdateUserUseCase } from "../application/update-user.usecase";
import { SqlUserRepository } from "../infrastructure/sql-user.repository";
import { UpdateUserBody, UserIdParam } from "./user.dto";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { UnauthorizedError } from "../../auth/domain/errors";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { tokenService } from "../../auth/infrastructure/services/token.service";

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

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(), tokenService);

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
    .error(ConflictError, ({ set, error }) => {
      set.status = 409;
      return { error: error.message };
    })
    .error(NotFoundError, ({ set, error }) => {
      set.status = 404;
      return { error: error.message };
    })

    .get("/", { detail: { summary: "List all profiles" } }, async () => listUsers.execute())

    .get(
      "/:id",
      { params: UserIdParam, detail: { summary: "Get a profile by ID" } },
      async ({ params }) => getUser.execute(params.id),
    )

    .patch(
      "/me",
      {
        body: UpdateUserBody,
        beforeHandle: [requireAuth],
        detail: { summary: "Update my profile" },
      },
      async ({ account, body }) => updateUser.execute(account!.id, body),
    )

    .get(
      "/me",
      {
        beforeHandle: [requireAuth],
        detail: { summary: "Get my profile" },
      },
      async ({ account }) => {
        const user = await repository.findByAccountId(account!.id);
        if (!user) throw new NotFoundError("User", account!.id);
        return user;
      },
    );
}
