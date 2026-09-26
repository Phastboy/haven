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
  fulfillmentSchema,
} from "../domain/fulfillment.schema";
import { t } from "elysia";

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
    .get("/", 
      {
        response: {
          200: t.Object({ data: fulfillmentSchema })
        }
      },
      async ({ params: { orderId }, session, set }) => {
      requireAuth({ session, set });

      // Ideally we would also verify if the user has access to this order (owner or requester)
      const fulfillment = await repository.getFulfillmentByOrderId(orderId);
      if (!fulfillment) {
        set.status = 404;
        return { error: "Fulfillment not found." };
      }
      return { data: fulfillment } as any;
    })
    .post(
      "/deliver",
      {
        body: deliverFulfillmentBodySchema,
        response: {
          200: t.Object({ data: fulfillmentSchema }) // or 201 depending on the logic, let's use 200 since it returns fulfillment
        }
      },
      async ({ params: { orderId }, body, user, session, set }) => {
        requireAuth({ session, set });

        if (!user) {
          throw new UnauthorizedError("User profile not found.");
        }

        const fulfillment = await deliverFulfillment.execute({
          orderId,
          accountId: user.id,
          ...(body.message && { deliveryMessage: body.message }),
          autoReviewDays: 3, // Default auto-complete threshold
        });
        return { data: fulfillment } as any;
      },
    )
    .post("/accept", 
      {
        response: {
          200: t.Object({ data: fulfillmentSchema })
        }
      },
      async ({ params: { orderId }, user, session, set }) => {
      requireAuth({ session, set });

      if (!user) {
        throw new UnauthorizedError("User profile not found.");
      }

      const fulfillment = await acceptFulfillment.execute({
        orderId,
        accountId: user.id,
      });
      return { data: fulfillment } as any;
    })
    .post(
      "/request-revision",
      {
        body: requestRevisionBodySchema,
        response: {
          200: t.Object({ data: fulfillmentSchema })
        }
      },
      async ({ params: { orderId }, body, user, session, set }) => {
        requireAuth({ session, set });

        if (!user) {
          throw new UnauthorizedError("User profile not found.");
        }

        const fulfillment = await requestRevision.execute({
          orderId,
          accountId: user.id,
          reason: body.reason,
        });
        return { data: fulfillment } as any;
      },
    );
};
