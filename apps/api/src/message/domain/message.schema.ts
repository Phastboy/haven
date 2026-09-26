import { t } from "elysia";

export const createThreadBodySchema = t.Object({
  participantId: t.String({ format: "uuid" }),
});

export const sendMessageBodySchema = t.Object({
  content: t.String({ minLength: 1, maxLength: 5000 }),
});

export const ThreadResponse = t.Object({
  id: t.String({ format: "uuid" }),
  participant1Id: t.String({ format: "uuid" }),
  participant2Id: t.String({ format: "uuid" }),
  createdAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
  updatedAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
});

export const MessageResponse = t.Object({
  id: t.String({ format: "uuid" }),
  threadId: t.String({ format: "uuid" }),
  senderId: t.String({ format: "uuid" }),
  content: t.String(),
  readAt: t.Optional(t.Union([t.String({ format: "date-time" }), t.Date(), t.Null()])),
  createdAt: t.Union([t.String({ format: "date-time" }), t.Date()]),
});
