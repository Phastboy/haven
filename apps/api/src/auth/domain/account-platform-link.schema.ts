import type { Static } from "typebox";
import { Type } from "typebox";

export const AccountPlatformLinkSchema = Type.Object({
  id: Type.String({ format: "uuid" }),
  accountId: Type.String({ format: "uuid" }),
  platformUserId: Type.String(),
  platform: Type.String(),
  createdAt: Type.String({ format: "date-time" }),
});

export type AccountPlatformLink = Static<typeof AccountPlatformLinkSchema>;
