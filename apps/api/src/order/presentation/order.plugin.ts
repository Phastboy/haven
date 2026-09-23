import { Elysia } from "elysia";
import { SqlOrderRepository } from "../infrastructure/sql-order.repository";
import { OfferAdapterService } from "../infrastructure/offer-adapter.service";
import { CreateOrderUseCase } from "../application/create-order.usecase";
import { UpdateOrderStatusUseCase } from "../application/update-order-status.usecase";
import { GetOrdersUseCase } from "../application/get-orders.usecase";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { tokenService } from "../../auth/infrastructure/services/token.service";
import { UnauthorizedError } from "../../auth/domain/errors";
import { db } from "../../database/db";
import { users } from "../../database/schema";
import { eq } from "drizzle-orm";
import { createOrderBodySchema, updateOrderStatusBodySchema } from "../domain/order.schema";
import {
  OrderNotFoundError,
  UnauthorizedOrderActionError,
  InvalidOrderStateTransitionError,
  SelfOrderNotAllowedError,
  OfferNotActiveError,
  OfferNotFoundError,
  DuplicateOrderError,
} from "../domain/errors";

import type { IEventBus } from "../../shared/domain/event-bus.interface";

export const createOrderPlugin = (eventBus?: IEventBus) => {
  const repository = new SqlOrderRepository();
  const offerAdapter = new OfferAdapterService();

  const createOrderUseCase = new CreateOrderUseCase(repository, offerAdapter);
  const updateOrderStatusUseCase = new UpdateOrderStatusUseCase(repository, offerAdapter, eventBus);
  const getOrdersUseCase = new GetOrdersUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(), tokenService);

  return new Elysia({ prefix: "/orders", tags: ["Orders"] })
    .error(({ error, set }) => {
      if (error instanceof OrderNotFoundError || error.name === "OrderNotFoundError") {
        set.status = 404;
        return { error: error.message };
      }
      if (
        error instanceof UnauthorizedOrderActionError ||
        error.name === "UnauthorizedOrderActionError"
      ) {
        set.status = 403;
        return { error: error.message };
      }
      if (
        error instanceof InvalidOrderStateTransitionError ||
        error.name === "InvalidOrderStateTransitionError"
      ) {
        set.status = 400;
        return { error: error.message };
      }
      if (error instanceof SelfOrderNotAllowedError || error.name === "SelfOrderNotAllowedError") {
        set.status = 400;
        return { error: error.message };
      }
      if (error instanceof OfferNotActiveError || error.name === "OfferNotActiveError") {
        set.status = 400;
        return { error: error.message };
      }
      if (error instanceof OfferNotFoundError || error.name === "OfferNotFoundError") {
        set.status = 404;
        return { error: error.message };
      }
      if (error instanceof DuplicateOrderError || error.name === "DuplicateOrderError") {
        set.status = 409;
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
    .post(
      "/",
      {
        body: createOrderBodySchema,
      },
      async ({ body, user, session, set }) => {
        const authCheck = requireAuth({ session, set });
        if (authCheck) return authCheck;

        if (!user) {
          set.status = 404;
          return { error: "User profile not found." };
        }

        const order = await createOrderUseCase.execute({
          offerId: body.offerId,
          requesterId: user.id,
          quantity: body.quantity || 1,
          ...(body.message && { message: body.message }),
        });
        set.status = 201;
        return order;
      },
    )
    .get("/me", async ({ user, session, set }) => {
      const authCheck = requireAuth({ session, set });
      if (authCheck) return authCheck;

      if (!user) {
        set.status = 404;
        return { error: "User profile not found." };
      }

      return getOrdersUseCase.getRequesterOrders(user.id);
    })
    .get("/received", async ({ user, session, set }) => {
      const authCheck = requireAuth({ session, set });
      if (authCheck) return authCheck;

      if (!user) {
        set.status = 404;
        return { error: "User profile not found." };
      }

      return getOrdersUseCase.getReceivedOrders(user.id);
    })
    .patch(
      "/:id/status",
      {
        body: updateOrderStatusBodySchema,
      },
      async ({ params, body, user, session, set }) => {
        const authCheck = requireAuth({ session, set });
        if (authCheck) return authCheck;

        return updateOrderStatusUseCase.execute({
          orderId: params.id,
          accountId: user!.id,
          newStatus: body.status,
        });
      },
    );
};
