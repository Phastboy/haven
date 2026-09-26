import { config } from "../../config";
import { Elysia } from "elysia";
import { SqlOrderRepository } from "../infrastructure/sql-order.repository";
import { OfferAdapterService } from "../infrastructure/offer-adapter.service";
import { CreateOrderUseCase } from "../application/create-order.usecase";
import { UpdateOrderStatusUseCase } from "../application/update-order-status.usecase";
import { GetOrdersUseCase } from "../application/get-orders.usecase";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { TokenService } from "../../auth/infrastructure/services/token.service";
const tokenService = new TokenService(config);
import { UnauthorizedError } from "../../auth/domain/errors";
import type { DB } from "../../database/db";
import { users } from "../../database/schema";
import { eq } from "drizzle-orm";
import { createOrderBodySchema, updateOrderStatusBodySchema, orderSchema } from "../domain/order.schema";
import { t } from "elysia";
import { PaginatedResponseSchema } from "../../shared/domain/pagination";

import type { IEventBus } from "../../shared/domain/event-bus.interface";

export const createOrderPlugin = (eventBus: IEventBus | undefined, db: DB) => {
  const repository = new SqlOrderRepository(db);
  const offerAdapter = new OfferAdapterService(db);

  const createOrderUseCase = new CreateOrderUseCase(repository, offerAdapter);
  const updateOrderStatusUseCase = new UpdateOrderStatusUseCase(repository, offerAdapter, eventBus);
  const getOrdersUseCase = new GetOrdersUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(db), tokenService);

  return new Elysia({ prefix: "/orders", tags: ["Orders"] })
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
    .post(
      "/",
      {
        body: createOrderBodySchema,
        response: {
          201: t.Object({ data: orderSchema })
        }
      },
      async ({ body, user, session, set }) => {
        requireAuth({ session, set });

        if (!user) {
          throw new UnauthorizedError("User profile not found.");
        }

        const order = await createOrderUseCase.execute({
          offerId: body.offerId,
          requesterId: user.id,
          quantity: body.quantity || 1,
          ...(body.message && { message: body.message }),
        });
        set.status = 201;
        return { data: order } as any;
      },
    )
    .get("/me", 
      {
        response: {
          200: PaginatedResponseSchema(t.Any()) // Using t.Any() here because it might include joined tables (like offer details). If strictly order, use orderSchema
        }
      },
      async ({ user, session, set }) => {
      requireAuth({ session, set });

      if (!user) {
        throw new UnauthorizedError("User profile not found.");
      }

      return await getOrdersUseCase.getRequesterOrders(user.id);
    })
    .get("/received", 
      {
        response: {
          200: PaginatedResponseSchema(t.Any())
        }
      },
      async ({ user, session, set }) => {
      requireAuth({ session, set });

      if (!user) {
        throw new UnauthorizedError("User profile not found.");
      }

      return await getOrdersUseCase.getReceivedOrders(user.id);
    })
    .patch(
      "/:orderId/status",
      {
        body: updateOrderStatusBodySchema,
        response: {
          200: t.Object({ data: orderSchema })
        }
      },
      async ({ params, body, user, session, set }) => {
        requireAuth({ session, set });

        const order = await updateOrderStatusUseCase.execute({
          orderId: params.orderId,
          accountId: user!.id,
          newStatus: body.status,
        });
        return { data: order } as any;
      },
    );
};
