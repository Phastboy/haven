import { SessionWithAccount } from "../../domain/session.schema";
export const requireAuth = (context: {
  session?: SessionWithAccount | null;
  set: { status?: number | string; [key: string]: unknown };
}) => {
  if (!context.session) {
    context.set.status = 401;
    return { message: "Unauthorized access" };
  }
  return;
};
