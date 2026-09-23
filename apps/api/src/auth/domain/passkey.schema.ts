import type { Static } from "typebox";
import { Type } from "typebox";

// Phase 2 Schema (Documented Only, Not Migrated)
export const PasskeySchema = Type.Object({
  id: Type.String({ format: "uuid" }),
  accountId: Type.String({ format: "uuid" }),
  credentialId: Type.String(),
  publicKey: Type.Any(), // bytes in DB
  counter: Type.Integer(),
  deviceName: Type.Optional(Type.String()),
  createdAt: Type.String({ format: "date-time" }),
  lastUsedAt: Type.Optional(Type.String({ format: "date-time" })),
});

export type Passkey = Static<typeof PasskeySchema>;
