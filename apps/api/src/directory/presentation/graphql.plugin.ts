import type { DB } from "../../database/db";
import { yoga } from "@elysia/graphql-yoga";
import { typeDefs } from "./graphql/schema";
import { resolvers } from "./graphql/resolvers";
import { SqlDirectoryRepository } from "../infrastructure/sql-directory.repository";
import type { GraphQLContext } from "./graphql/context";
import DataLoader from "dataloader";

export const createDirectoryPlugin = (db: DB) => {
  const repository = new SqlDirectoryRepository(db);

  return yoga({
    typeDefs,
    resolvers: resolvers as never,
    context: (): GraphQLContext => {
      const userLoader = new DataLoader(async (userIds: readonly string[]) => {
        const users = await repository.getUsersByIds([...userIds]);
        const userMap = new Map(users.map((u) => [u.id, u]));
        return userIds.map((id) => userMap.get(id) || null);
      });

      return {
        repository,
        userLoader,
      };
    },
  });
};
