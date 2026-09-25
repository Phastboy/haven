import { config } from "../../config";
import { Elysia, t } from "elysia";
import { websocket } from "elysia/websocket";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { TokenService } from "../../auth/infrastructure/services/token.service";
const tokenService = new TokenService(config);
import type { DB } from "../../database/db";
import { users } from "../../database/schema";
import { eq } from "drizzle-orm";
import { UnauthorizedError } from "../../auth/domain/errors";

export const createWsPlugin = (db: DB) => {
  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(db), tokenService);

  return new Elysia().use(websocket()).ws("/ws", {
    query: t.Object({
      token: t.String(),
    }),
    async open(ws) {
      const { token } = (ws as unknown as { data: { query: { token: string } } }).data.query;
      try {
        const sessionWithAccount = await getSessionUseCase.execute(token);
        const [user] = await db
          .select()
          .from(users)
          .where(eq(users.accountId, sessionWithAccount.account.id));

        if (!user) {
          ws.close();
          return;
        }

        // Subscribe to user-specific channel
        ws.subscribe(`user:${user.id}`);
        console.log(`[WebSocket] User ${user.id} connected and subscribed to user:${user.id}`);
      } catch (e) {
        if (e instanceof UnauthorizedError) {
          ws.close();
        } else {
          console.error("[WebSocket] Authentication error:", e);
          ws.close();
        }
      }
    },
    message(_ws, _message) {
      // In this version, we primarily use HTTP for pushing state and WS for reading.
      // But we can echo or handle custom events here if needed.
    },
    close(_ws) {
      // Cleanup happens automatically, Bun WS automatically unsubscribes on close
    },
  });
};
