import { config } from "../../config";
import { Elysia } from "elysia";
import { SqlFulfillmentRepository } from "../infrastructure/sql-fulfillment.repository";
import { OrderFulfillmentAdapter } from "../infrastructure/order-fulfillment.adapter";
import { DeliverFulfillmentUseCase } from "../application/deliver-fulfillment.usecase";
import { AcceptFulfillmentUseCase } from "../application/accept-fulfillment.usecase";
import { RequestRevisionUseCase } from "../application/request-revision.usecase";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { TokenService } from "../../auth/infrastructure/services/token.service";
const tokenService = new TokenService(config);
import { UnauthorizedError } from "../../auth/domain/errors";
import type { DB } from "../../database/db";
import { users } from "../../database/schema";
import { eq } from "drizzle-orm";
import {
  deliverFulfillmentBodySchema,
  requestRevisionBodySchema,
} from "../domain/fulfillment.schema";

export const createFulfillmentPlugin = (db: DB) => {
  const repository = new SqlFulfillmentRepository(db);
  const adapter = new OrderFulfillmentAdapter(db);

  const deliverFulfillment = new DeliverFulfillmentUseCase(repository, adapter);
  const acceptFulfillment = new AcceptFulfillmentUseCase(repository, adapter);
  const requestRevision = new RequestRevisionUseCase(repository, adapter);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(db), tokenService);

  return new Elysia({ prefix: "/orders/:orderId/fulfillment", tags: ["Fulfillment"] })
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
        if (e instanceof UnauthorizedError) return { session: null, account: null, user: null };
        throw e;
      }
    })
    .get("/", async ({ params: { orderId }, session, set }) => {
      requireAuth({ session, set });

      // Ideally we would also verify if the user has access to this order (owner or requester)
      const fulfillment = await repository.getFulfillmentByOrderId(orderId);
      if (!fulfillment) {
        set.status = 404;
        return { error: "Fulfillment not found." };
      }
      return { data: fulfillment };
    })
    .post(
      "/deliver",
      {
        body: deliverFulfillmentBodySchema,
      },
      async ({ params: { orderId }, body, user, session, set }) => {
        requireAuth({ session, set });

        if (!user) {
          set.status = 404;
          return { error: "User profile not found." };
        }

        const fulfillment = await deliverFulfillment.execute({
          orderId,
          accountId: user.id,
          ...(body.message && { deliveryMessage: body.message }),
          autoReviewDays: 3, // Default auto-complete threshold
        });
        return { data: fulfillment };
      },
    )
    .post("/accept", async ({ params: { orderId }, user, session, set }) => {
      requireAuth({ session, set });

      if (!user) {
        set.status = 404;
        return { error: "User profile not found." };
      }

      const fulfillment = await acceptFulfillment.execute({
        orderId,
        accountId: user.id,
      });
      return { data: fulfillment };
    })
    .post(
      "/request-revision",
      {
        body: requestRevisionBodySchema,
      },
      async ({ params: { orderId }, body, user, session, set }) => {
        requireAuth({ session, set });

        if (!user) {
          set.status = 404;
          return { error: "User profile not found." };
        }

        const fulfillment = await requestRevision.execute({
          orderId,
          accountId: user.id,
          reason: body.reason,
        });
        return { data: fulfillment };
      },
    );
};
