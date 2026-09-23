import type { SessionWithAccount } from "../../domain/session.schema";
import { UnauthorizedError } from "../../domain/errors";

export const requireAuth = (context: {
  session?: SessionWithAccount | null;
  set: { status?: number | string; [key: string]: unknown };
}) => {
  if (!context.session) {
    throw new UnauthorizedError();
  }
  return;
};
