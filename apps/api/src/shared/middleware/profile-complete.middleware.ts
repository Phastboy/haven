import type { User } from "../../user/domain/user.schema";
import { ProfileIncompleteError } from "../errors";

export const requireProfileComplete = ({ user }: { user: User | null; set?: unknown }) => {
  if (!user || !user.profileComplete) {
    throw new ProfileIncompleteError();
  }
};
