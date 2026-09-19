import { t } from "elysia";
import type { Static } from "typebox";

export const fulfillmentStatusSchema = t.Union([
  t.Literal("PENDING"),
  t.Literal("DELIVERED"),
  t.Literal("REVISION_REQUESTED"),
  t.Literal("COMPLETED"),
]);

export const fulfillmentSchema = t.Object({
  id: t.String(),
  orderId: t.String(),
  status: fulfillmentStatusSchema,
  deliveryMessage: t.Union([t.String(), t.Null()]),
  reviewDeadline: t.Union([t.Date(), t.Null()]),
  createdAt: t.Date(),
  updatedAt: t.Date(),
});

export const deliverFulfillmentBodySchema = t.Object({
  message: t.Optional(t.String()),
});

export const requestRevisionBodySchema = t.Object({
  reason: t.String(),
});

export type FulfillmentStatus = Static<typeof fulfillmentStatusSchema>;
export type Fulfillment = Static<typeof fulfillmentSchema>;
