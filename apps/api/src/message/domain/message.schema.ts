import { t } from "elysia";

export const createThreadBodySchema = t.Object({
  participantId: t.String({ format: "uuid" }),
});

export const sendMessageBodySchema = t.Object({
  content: t.String({ minLength: 1, maxLength: 5000 }),
});
