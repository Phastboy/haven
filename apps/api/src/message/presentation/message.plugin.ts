import { Elysia } from "elysia";
import { SqlMessageRepository } from "../infrastructure/sql-message.repository";
import { CreateThreadUseCase } from "../application/create-thread.usecase";
import { SendMessageUseCase } from "../application/send-message.usecase";
import { GetThreadsUseCase } from "../application/get-threads.usecase";
import { GetMessagesUseCase } from "../application/get-messages.usecase";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { tokenService } from "../../auth/infrastructure/services/token.service";
import { UnauthorizedError } from "../../auth/domain/errors";
import { db } from "../../database/db";
import { users } from "../../database/schema";
import { eq } from "drizzle-orm";
import { createThreadBodySchema, sendMessageBodySchema } from "../domain/message.schema";
import {
  ThreadNotFoundError,
  UnauthorizedThreadAccessError,
  ParticipantNotFoundError,
  SelfThreadError,
} from "../domain/errors";

import type { IEventBus } from "../../shared/domain/event-bus.interface";

export const createMessagePlugin = (eventBus: IEventBus) => {
  const repository = new SqlMessageRepository();
  const createThreadUseCase = new CreateThreadUseCase(repository);
  const sendMessageUseCase = new SendMessageUseCase(repository, eventBus);
  const getThreadsUseCase = new GetThreadsUseCase(repository);
  const getMessagesUseCase = new GetMessagesUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(), tokenService);

  return new Elysia({ prefix: "/messages", tags: ["Messages"] })
    .error(({ error, set }) => {
      if (error instanceof ThreadNotFoundError || error.name === "ThreadNotFoundError") {
        set.status = 404;
        return { error: error.message };
      }
      if (
        error instanceof UnauthorizedThreadAccessError ||
        error.name === "UnauthorizedThreadAccessError"
      ) {
        set.status = 403;
        return { error: error.message };
      }
      if (error instanceof ParticipantNotFoundError || error.name === "ParticipantNotFoundError") {
        set.status = 404;
        return { error: error.message };
      }
      if (error instanceof SelfThreadError || error.name === "SelfThreadError") {
        set.status = 400;
        return { error: error.message };
      }
      return;
    })
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
    .get("/threads", async ({ user, session, set }) => {
      const authCheck = requireAuth({ session, set });
      if (authCheck) return authCheck;

      if (!user) {
        set.status = 404;
        return { error: "User profile not found." };
      }

      return getThreadsUseCase.execute(user.id);
    })
    .post(
      "/threads",
      {
        body: createThreadBodySchema,
      },
      async ({ body, user, session, set }) => {
        const authCheck = requireAuth({ session, set });
        if (authCheck) return authCheck;

        if (!user) {
          set.status = 404;
          return { error: "User profile not found." };
        }

        const thread = await createThreadUseCase.execute(user.id, body.participantId);
        set.status = 201;
        return thread;
      },
    )
    .get("/threads/:threadId", async ({ params, user, session, set }) => {
      const authCheck = requireAuth({ session, set });
      if (authCheck) return authCheck;

      if (!user) {
        set.status = 404;
        return { error: "User profile not found." };
      }

      return getMessagesUseCase.execute(params.threadId, user.id);
    })
    .post(
      "/threads/:threadId",
      {
        body: sendMessageBodySchema,
      },
      async ({ params, body, user, session, set }) => {
        const authCheck = requireAuth({ session, set });
        if (authCheck) return authCheck;

        if (!user) {
          set.status = 404;
          return { error: "User profile not found." };
        }

        const message = await sendMessageUseCase.execute(params.threadId, user.id, body.content);
        set.status = 201;
        return message;
      },
    );
};
