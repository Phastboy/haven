import { Elysia } from 'elysia';
import { SqlOrderRepository } from '../infrastructure/sql-order.repository';
import { OfferAdapterService } from '../infrastructure/offer-adapter.service';
import { CreateOrderUseCase } from '../application/create-order.usecase';
import { UpdateOrderStatusUseCase } from '../application/update-order-status.usecase';
import { GetOrdersUseCase } from '../application/get-orders.usecase';
import { requireAuth } from '../../auth/presentation/middleware/session.middleware';
import { GetSessionUseCase } from '../../auth/application/use-cases/get-session.use-case';
import { SessionRepository } from '../../auth/infrastructure/repositories/session.repository';
import { tokenService } from '../../auth/infrastructure/services/token.service';
import { UnauthorizedError } from '../../auth/domain/errors';
import { db } from '../../database/db';
import { users } from '../../database/schema';
import { eq } from 'drizzle-orm';
import { createOrderBodySchema, updateOrderStatusBodySchema } from '../domain/order.schema';
import { OrderNotFoundError, UnauthorizedOrderActionError, InvalidOrderStateTransitionError, SelfOrderNotAllowedError, OfferNotActiveError } from '../domain/errors';

export const createOrderPlugin = () => {
  const repository = new SqlOrderRepository();
  const offerAdapter = new OfferAdapterService();
  const createOrderUseCase = new CreateOrderUseCase(repository, offerAdapter);
  const updateOrderStatusUseCase = new UpdateOrderStatusUseCase(repository, offerAdapter);
  const getOrdersUseCase = new GetOrdersUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(
    new SessionRepository(),
    tokenService
  );

  return new Elysia({ prefix: '/orders', tags: ['Orders'] })
    .error(OrderNotFoundError, ({ set, error }) => { set.status = 404; return { error: error.message }; })
    .error(UnauthorizedOrderActionError, ({ set, error }) => { set.status = 403; return { error: error.message }; })
    .error(InvalidOrderStateTransitionError, ({ set, error }) => { set.status = 400; return { error: error.message }; })
    .error(SelfOrderNotAllowedError, ({ set, error }) => { set.status = 400; return { error: error.message }; })
    .error(OfferNotActiveError, ({ set, error }) => { set.status = 400; return { error: error.message }; })
    .derive(async ({ headers }: { headers: Record<string, string | undefined> }) => {
      const authHeader = headers['authorization'];
      if (!authHeader || !authHeader.startsWith('Bearer ')) {
        return { session: null, account: null, user: null };
      }

      const token = authHeader.substring(7);
      try {
        const sessionWithAccount = await getSessionUseCase.execute(token);
        const [user] = await db.select().from(users).where(eq(users.accountId, sessionWithAccount.account.id));
        return { session: sessionWithAccount, account: sessionWithAccount.account, user };
      } catch (e: unknown) {
        if (e instanceof UnauthorizedError) {
          return { session: null, account: null, user: null };
        }
        throw e;
      }
    })
    .post('/', {
      body: createOrderBodySchema,
    }, async ({ body, user, session, set }) => {
      const authCheck = requireAuth({ session, set } as any);
      if (authCheck) return authCheck;

      const order = await createOrderUseCase.execute({
        offerId: body.offerId,
        requesterId: user!.id,
        quantity: body.quantity || 1,
        message: body.message,
      });
      set.status = 201;
      return order;
    })
    .get('/me', async ({ user, session, set }) => {
      const authCheck = requireAuth({ session, set } as any);
      if (authCheck) return authCheck;
      
      return getOrdersUseCase.getRequesterOrders(user!.id);
    })
    .get('/received', async ({ user, session, set }) => {
      const authCheck = requireAuth({ session, set } as any);
      if (authCheck) return authCheck;

      return getOrdersUseCase.getReceivedOrders(user!.id);
    })
    .patch('/:id/status', {
      body: updateOrderStatusBodySchema,
    }, async ({ params, body, user, session, set }) => {
      const authCheck = requireAuth({ session, set } as any);
      if (authCheck) return authCheck;

      return updateOrderStatusUseCase.execute({
        orderId: params.id,
        accountId: user!.id,
        newStatus: body.status,
      });
    });
};
