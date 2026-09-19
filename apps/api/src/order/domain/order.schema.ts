import { t } from "elysia";
import type { Static } from "@sinclair/typebox";

export const orderStatusSchema = t.Union([
  t.Literal("PENDING"),
  t.Literal("ACCEPTED"),
  t.Literal("REJECTED"),
  t.Literal("CANCELLED"),
  t.Literal("COMPLETED"),
]);

export const orderSchema = t.Object({
  id: t.String(),
  offerId: t.String(),
  requesterId: t.String(),
  price: t.Number(),
  quantity: t.Number(),
  status: orderStatusSchema,
  message: t.Union([t.String(), t.Null()]),
  createdAt: t.Date(),
  updatedAt: t.Date(),
});

export const createOrderBodySchema = t.Object({
  offerId: t.String(),
  quantity: t.Optional(t.Number({ minimum: 1 })),
  message: t.Optional(t.String()),
});

export const updateOrderStatusBodySchema = t.Object({
  status: orderStatusSchema,
});

export type OrderStatus = Static<typeof orderStatusSchema>;
export type Order = Static<typeof orderSchema>;
