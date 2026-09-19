import { t } from "elysia";
import type { Static } from "@sinclair/typebox";
import { CreateOfferDataSchema, UpdateOfferDataSchema } from "../domain/offer.schema";

export const CreateOfferBody = CreateOfferDataSchema;
export const UpdateOfferBody = UpdateOfferDataSchema;

export const OfferIdParam = t.Object({
  id: t.String({ format: "uuid" }),
});

export const UserIdParam = t.Object({
  userId: t.String({ format: "uuid" }),
});
