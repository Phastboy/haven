import { config } from "../../config";
import { Elysia } from "elysia";
import { CreateOfferBody, UpdateOfferBody, OfferIdParam, UserIdParam } from "./offer.dto";
import { OfferSchema } from "../domain/offer.schema";
import { t } from "elysia";
import { PaginatedResponseSchema } from "../../shared/domain/pagination";
import { SqlOfferRepository } from "../infrastructure/sql-offer.repository";
import { CreateOfferUseCase } from "../application/create-offer.usecase";
import { UpdateOfferUseCase } from "../application/update-offer.usecase";
import { DeleteOfferUseCase } from "../application/delete-offer.usecase";
import { GetOfferUseCase } from "../application/get-offer.usecase";
import { ListUserOffersUseCase } from "../application/list-user-offers.usecase";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { requireProfileComplete } from "../../shared/middleware/profile-complete.middleware";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { TokenService } from "../../auth/infrastructure/services/token.service";
const tokenService = new TokenService(config);
import { UnauthorizedError } from "../../auth/domain/errors";
import { OfferNotFoundError } from "../domain/errors";
import type { DB } from "../../database/db";
import { SqlUserRepository } from "../../user/infrastructure/sql-user.repository";

export const createOfferPlugin = (db: DB) => {
  const repository = new SqlOfferRepository(db);
  const createOffer = new CreateOfferUseCase(repository);
  const updateOffer = new UpdateOfferUseCase(repository);
  const deleteOffer = new DeleteOfferUseCase(repository);
  const getOffer = new GetOfferUseCase(repository);
  const listUserOffers = new ListUserOffersUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(db), tokenService);

  return new Elysia({ prefix: "/offers", tags: ["Offers"] })
    .derive(async ({ headers }: { headers: Record<string, string | undefined> }) => {
      const authHeader = headers["authorization"];
      if (!authHeader || !authHeader.startsWith("Bearer ")) {
        return { session: null, account: null, user: null };
      }

      const token = authHeader.substring(7);
      try {
        const sessionWithAccount = await getSessionUseCase.execute(token);
        const userRepo = new SqlUserRepository(db);
        const user = await userRepo.findByAccountId(sessionWithAccount.account.id);
        return {
          session: sessionWithAccount,
          account: sessionWithAccount.account,
          user,
        };
      } catch (e: unknown) {
        if (e instanceof UnauthorizedError) {
          return { session: null, account: null, user: null };
        }
        throw e;
      }
    })
    .get(
      "/:id",
      {
        params: OfferIdParam,
        response: {
          200: t.Object({ data: OfferSchema }),
        },
      },
      async ({ params, user }) => {
        const offer = await getOffer.execute(params.id);
        // Archived offers are invisible to everyone except their owner.
        if (offer.status === "ARCHIVED" && offer.userId !== user?.id) {
          throw new OfferNotFoundError();
        }
        return { data: offer } as never;
      },
    )
    .get(
      "/user/:userId",
      {
        params: UserIdParam,
        query: t.Object({
          page:  t.Optional(t.Numeric({ minimum: 1, default: 1 })),
          limit: t.Optional(t.Numeric({ minimum: 1, maximum: 100, default: 50 })),
        }),
        response: {
          200: PaginatedResponseSchema(OfferSchema),
        },
      },
      async ({ params, user, query }) => {
        const limit  = query.limit  ?? 50;
        const offset = ((query.page ?? 1) - 1) * limit;
        return listUserOffers.execute(params.userId, user?.id, limit, offset);
      },
    )
    .post(
      "/",
      {
        body: CreateOfferBody,
        response: {
          201: t.Object({ data: OfferSchema }),
        },
      },
      async ({ body, user, session, set }) => {
        requireAuth({ session, set });
        requireProfileComplete({ user, set });

        if (!user) {
          throw new UnauthorizedError("User profile not found.");
        }

        const offer = await createOffer.execute(user.id, body);
        set.status = 201;
        return { data: offer } as never;
      },
    )
    .patch(
      "/:id",
      {
        params: OfferIdParam,
        body: UpdateOfferBody,
        response: {
          200: t.Object({ data: OfferSchema }),
        },
      },
      async ({ params, body, user, session, set }) => {
        requireAuth({ session, set });

        const offer = await updateOffer.execute(user!.id, params.id, body);
        return { data: offer } as never;
      },
    )
    .delete(
      "/:id",
      {
        params: OfferIdParam,
      },
      async ({ params, user, session, set }) => {
        requireAuth({ session, set });

        await deleteOffer.execute(user!.id, params.id);
        set.status = 204;
        return;
      },
    );
};
