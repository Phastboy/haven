import { Elysia } from "elysia";
import { CreateOfferBody, UpdateOfferBody, OfferIdParam, UserIdParam } from "./offer.dto";
import { SqlOfferRepository } from "../infrastructure/sql-offer.repository";
import { CreateOfferUseCase } from "../application/create-offer.usecase";
import { UpdateOfferUseCase } from "../application/update-offer.usecase";
import { DeleteOfferUseCase } from "../application/delete-offer.usecase";
import { GetOfferUseCase } from "../application/get-offer.usecase";
import { ListUserOffersUseCase } from "../application/list-user-offers.usecase";
import { OfferNotFoundError, UnauthorizedOfferActionError } from "../domain/errors";
import { requireAuth } from "../../auth/presentation/middleware/session.middleware";
import { GetSessionUseCase } from "../../auth/application/use-cases/get-session.use-case";
import { SessionRepository } from "../../auth/infrastructure/repositories/session.repository";
import { tokenService } from "../../auth/infrastructure/services/token.service";
import { UnauthorizedError } from "../../auth/domain/errors";
import { db } from "../../database/db";
import { users } from "../../database/schema";
import { eq } from "drizzle-orm";

export const createOfferPlugin = () => {
  const repository = new SqlOfferRepository();
  const createOffer = new CreateOfferUseCase(repository);
  const updateOffer = new UpdateOfferUseCase(repository);
  const deleteOffer = new DeleteOfferUseCase(repository);
  const getOffer = new GetOfferUseCase(repository);
  const listUserOffers = new ListUserOffersUseCase(repository);

  const getSessionUseCase = new GetSessionUseCase(new SessionRepository(), tokenService);

  return new Elysia({ prefix: "/offers", tags: ["Offers"] })
    .error(({ error, set }) => {
      if (error instanceof OfferNotFoundError || error.name === "OfferNotFoundError") {
        set.status = 404;
        return { error: error.message };
      }
      if (
        error instanceof UnauthorizedOfferActionError ||
        error.name === "UnauthorizedOfferActionError"
      ) {
        set.status = 403;
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
      },
      async ({ params, user }) => {
        const offer = await getOffer.execute(params.id);
        // Archived offers are invisible to everyone except their owner.
        if (offer.status === "ARCHIVED" && offer.userId !== user?.id) {
          throw new OfferNotFoundError();
        }
        return offer;
      },
    )
    .get(
      "/user/:userId",
      {
        params: UserIdParam,
      },
      async ({ params, user }) => {
        // Pass the requester's user.id so the use-case can decide visibility.
        return listUserOffers.execute(params.userId, user?.id);
      },
    )
    .post(
      "/",
      {
        body: CreateOfferBody,
      },
      async ({ body, user, session, set }) => {
        const authCheck = requireAuth({ session, set });
        if (authCheck) return authCheck;

        if (!user) {
          set.status = 404;
          return { error: "User profile not found." };
        }

        const offer = await createOffer.execute(user.id, body);
        set.status = 201;
        return offer;
      },
    )
    .patch(
      "/:id",
      {
        params: OfferIdParam,
        body: UpdateOfferBody,
      },
      async ({ params, body, user, session, set }) => {
        const authCheck = requireAuth({ session, set });
        if (authCheck) return authCheck;

        return updateOffer.execute(user!.id, params.id, body);
      },
    )
    .delete(
      "/:id",
      {
        params: OfferIdParam,
      },
      async ({ params, user, session, set }) => {
        const authCheck = requireAuth({ session, set });
        if (authCheck) return authCheck;

        await deleteOffer.execute(user!.id, params.id);
        set.status = 204;
        return;
      },
    );
};
