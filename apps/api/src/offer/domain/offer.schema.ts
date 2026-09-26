import { t } from "elysia";
import type { Static } from "typebox";

export const OfferStatusSchema = t.Union([
  t.Literal("ACTIVE"),
  t.Literal("PAUSED"),
  t.Literal("ARCHIVED"),
]);
export type OfferStatus = Static<typeof OfferStatusSchema>;

export const OfferTypeSchema = t.Union([
  t.Literal("PRODUCT"),
  t.Literal("SERVICE"),
  t.Literal("APPOINTMENT"),
]);
export type OfferType = Static<typeof OfferTypeSchema>;

export const OfferSchema = t.Object({
  id: t.String({ format: "uuid" }),
  userId: t.String({ format: "uuid" }),
  title: t.String({ minLength: 1, maxLength: 255 }),
  description: t.Union([t.String(), t.Null()]),
  price: t.Integer({ minimum: 0 }),
  status: OfferStatusSchema,
  offerType: OfferTypeSchema,
  images: t.Union([t.Array(t.String({ format: "uri" })), t.Null()]),
  createdAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
  updatedAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
});
export type Offer = Static<typeof OfferSchema>;

export const CreateOfferDataSchema = t.Object({
  title: t.String({ minLength: 1, maxLength: 255 }),
  description: t.Optional(t.String()),
  price: t.Optional(t.Integer({ minimum: 0 })),
  offerType: t.Optional(OfferTypeSchema),
  images: t.Optional(t.Array(t.String({ format: "uri" }))),
});
export type CreateOfferData = Static<typeof CreateOfferDataSchema>;

export const UpdateOfferDataSchema = t.Partial(CreateOfferDataSchema);
export type UpdateOfferData = Static<typeof UpdateOfferDataSchema>;
