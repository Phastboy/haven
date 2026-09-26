import { config } from "../../config";
import { Elysia } from "elysia";
import { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import { CreateThreadUseCase } from "../application/create-thread.usecase";
import { SendMessageUseCase } from "../application/send-message.usecase";
import { GetThreadsUseCase } from "../application/get-threads.usecase";
import { GetMessagesUseCase } from "../application/get-messages.usecase";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { TokenService } from "../../auth/infrastructure/services/token.service";
const tokenService = new TokenService(config);
import { UnauthorizedError } from "../../auth/domain/errors";
import type { DB } from "../../database/db";
import { users } from "../../database/schema";
import { eq } from "drizzle-orm";
import { createThreadBodySchema, sendMessageBodySchema, ThreadResponse, MessageResponse } from "../domain/message.schema";
import { t } from "elysia";
import { PaginatedResponseSchema } from "../../shared/domain/pagination";

import type { IEventBus } from "../../shared/domain/event-bus.interface";

export const createMessagePlugin = (eventBus: IEventBus, db: DB) => {
  const repository = new SqlMessageRepository(db);
  const createThreadUseCase = new CreateThreadUseCase(repository);
  const sendMessageUseCase = new SendMessageUseCase(repository, eventBus);
  const getThreadsUseCase = new GetThreadsUseCase(repository);
  const getMessagesUseCase = new GetMessagesUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(db), tokenService);

  return new Elysia({ prefix: "/messages", tags: ["Messages"] })
    .derive(async ({ headers }: { headers: Record<string, string | undefined> }) => {
      const authHeader = headers["authorization"];
      if (!authHeader || !authHeader.startsWith("Bearer ")) {
        return { session: null, account: null, user: null };
      }

      const token = authHeader.substring(7);
      try {
        const sessionWithAccount = await getSessionUseCase.execute(token);
        const [user] = await db
          .select()
          .from(users)
          .where(eq(users.accountId, sessionWithAccount.account.id));
        return { session: sessionWithAccount, account: sessionWithAccount.account, user };
      } catch (e: unknown) {
        if (e instanceof UnauthorizedError) {
          return { session: null, account: null, user: null };
        }
        throw e;
      }
    })
    .get(
      "/threads",
      {
        response: {
          200: PaginatedResponseSchema(t.Any()), // Array of threads with latestMessage (complex type, t.Any() is fine to stop recursion if it's too nested, or just t.Array(t.Object({ ...ThreadResponse.properties, latestMessage: t.Any() })))
        }
      },
      async ({ user, session, set }) => {
      requireAuth({ session, set });

      if (!user) {
        throw new UnauthorizedError("User profile not found.");
      }

      return await getThreadsUseCase.execute(user.id);
    })
    .post(
      "/threads",
      {
        body: createThreadBodySchema,
        response: {
          201: t.Object({ data: ThreadResponse }),
        }
      },
      async ({ body, user, session, set }) => {
        requireAuth({ session, set });

        if (!user) {
          throw new UnauthorizedError("User profile not found.");
        }

        const thread = await createThreadUseCase.execute(user.id, body.participantId);
        set.status = 201;
        return { data: thread } as any;
      },
    )
    .get(
      "/threads/:threadId",
      {
        response: {
          200: PaginatedResponseSchema(MessageResponse),
        }
      },
      async ({ params, user, session, set }) => {
      requireAuth({ session, set });

      if (!user) {
        throw new UnauthorizedError("User profile not found.");
      }

      return await getMessagesUseCase.execute(params.threadId, user.id);
    })
    .post(
      "/threads/:threadId",
      {
        body: sendMessageBodySchema,
        response: {
          201: t.Object({ data: MessageResponse }),
        }
      },
      async ({ params, body, user, session, set }) => {
        requireAuth({ session, set });

        if (!user) {
          throw new UnauthorizedError("User profile not found.");
        }

        const message = await sendMessageUseCase.execute(params.threadId, user.id, body.content);
        set.status = 201;
        return { data: message } as any;
      },
    )
    .patch(
      "/threads/:threadId/read",
      {
        response: {
          200: t.Object({ success: t.Boolean() }),
        }
      },
      async ({ params, user, session, set }) => {
        requireAuth({ session, set });
        if (!user) {
          throw new UnauthorizedError("User profile not found.");
        }
        await repository.markMessagesAsRead(params.threadId, user.id);
        return { success: true };
      }
    );
};
