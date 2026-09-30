import { t } from "elysia";

export const createThreadBodySchema = t.Object({
  participantId: t.String({ format: "uuid" }),
});

export const sendMessageBodySchema = t.Object({
  content: t.String({ minLength: 1, maxLength: 5000 }),
  contextOfferIds: t.Optional(t.Array(t.String({ format: "uuid" }), { maxItems: 5 })),
});

export const ThreadResponse = t.Object({
  id: t.String({ format: "uuid" }),
  participant1Id: t.String({ format: "uuid" }),
  participant2Id: t.String({ format: "uuid" }),
  createdAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
  updatedAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
});

export const MessageContextResponse = t.Object({
  id: t.String({ format: "uuid" }),
  offerId: t.Union([t.String({ format: "uuid" }), t.Null()]),
});

export const MessageResponse = t.Object({
  id: t.String({ format: "uuid" }),
  threadId: t.String({ format: "uuid" }),
  senderId: t.String({ format: "uuid" }),
  content: t.String(),
  contexts: t.Optional(t.Array(MessageContextResponse)),
  readAt: t.Optional(t.Union([t.String({ format: "date-time" }), t.Date(), t.Null()])),
  createdAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
});
