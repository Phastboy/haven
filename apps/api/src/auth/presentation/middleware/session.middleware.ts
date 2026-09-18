import { Elysia, Context } from 'elysia';
import { SessionWithAccount } from '../../domain/session.schema';
export const requireAuth = (context: Context & { session?: SessionWithAccount | null }) => {
  if (!context.session) {
    context.set.status = 401;
    return { message: 'Unauthorized access' };
  }
  return;
};
