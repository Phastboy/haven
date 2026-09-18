import { Context } from 'elysia';
import { SessionWithAccount } from '../../domain/session.schema';

export const requireAuth = ({ session, set }: Context & { session?: SessionWithAccount | null }) => {
  if (!session) {
    set.status = 401;
    return { message: 'Unauthorized access' };
  }
};
