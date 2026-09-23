import type { GraphQLContext } from "./context";
import type { DirectoryOffer } from "../../domain/directory.schema";

export const resolvers = {
  Query: {
    activeOffers: async (
      _parent: unknown,
      args: { limit?: number; offset?: number },
      context: GraphQLContext,
    ): Promise<DirectoryOffer[]> => {
      return context.repository.getActiveOffers(args.limit, args.offset);
    },
  },
  Offer: {
    user: async (parent: DirectoryOffer, _args: unknown, context: GraphQLContext) => {
      return context.userLoader.load(parent.userId);
    },
    createdAt: (parent: DirectoryOffer) => parent.createdAt.toISOString(),
  },
};
