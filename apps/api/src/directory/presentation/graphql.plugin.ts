import { yoga } from '@elysia/graphql-yoga';
import { typeDefs } from './graphql/schema';
import { resolvers } from './graphql/resolvers';
import { SqlDirectoryRepository } from '../infrastructure/sql-directory.repository';
import { GraphQLContext } from './graphql/context';
import DataLoader from 'dataloader';

export const createDirectoryPlugin = () => {
  const repository = new SqlDirectoryRepository();

  return yoga({
    typeDefs,
    resolvers: resolvers as any,
    context: (): GraphQLContext => {
      const userLoader = new DataLoader(async (userIds: readonly string[]) => {
        const users = await repository.getUsersByIds([...userIds]);
        const userMap = new Map(users.map(u => [u.id, u]));
        return userIds.map(id => userMap.get(id) || null);
      });

      return {
        repository,
        userLoader
      };
    }
  });
};
