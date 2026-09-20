import openapi from "@elysia/openapi";
import { Elysia } from "elysia";
import { createUserPlugin } from "./user/presentation/user.plugin";
import { authController } from "./auth";
import { createOfferPlugin } from "./offer/presentation/offer.plugin";
import { createDirectoryPlugin } from "./directory/presentation/graphql.plugin";
import { createOrderPlugin } from "./order/presentation/order.plugin";
import { createFulfillmentPlugin } from "./fulfillment/presentation/fulfillment.plugin";
import { createAutoCompletePlugin } from "./scheduler/auto-complete.plugin";

import { configController } from "./config.controller";
import { createMessagePlugin } from "./message/presentation/message.plugin";
import { NodeEventEmitterAdapter } from "./shared/infrastructure/node-event-bus.adapter";
import { SqlMessageRepository } from "./message/infrastructure/sql-message.repository";
import { CreateThreadUseCase } from "./message/application/create-thread.usecase";

const eventBus = new NodeEventEmitterAdapter();

// Set up cross-domain event listeners
eventBus.subscribe("order.accepted", async (payload) => {
  const messageRepo = new SqlMessageRepository();
  const createThreadUseCase = new CreateThreadUseCase(messageRepo);
  try {
    await createThreadUseCase.execute(payload.requesterId, payload.ownerId);
    console.log(`[EventBus] Thread auto-created for order ${payload.orderId}`);
  } catch (error) {
    console.error(`[EventBus] Error creating thread for order ${payload.orderId}:`, error);
  }
});

export const app = new Elysia({ prefix: "/api" })
  .use(openapi())
  .use(configController)
  .use(createUserPlugin())
  .use(authController)
  .use(createOfferPlugin())
  .use(createDirectoryPlugin())
  .use(createOrderPlugin(eventBus))
  .use(createFulfillmentPlugin())
  .use(createMessagePlugin())
  .use(createAutoCompletePlugin())
  .request(({ set }) => {
    set.headers["Access-Control-Allow-Origin"] = "*";
    set.headers["Access-Control-Allow-Methods"] = "GET, POST, PUT, DELETE, OPTIONS";
    set.headers["Access-Control-Allow-Headers"] = "Content-Type, Authorization";
  })
  .options("/*", ({ set }: { set: { headers: Record<string, string> } }) => {
    set.headers["Access-Control-Allow-Origin"] = "*";
    set.headers["Access-Control-Allow-Methods"] = "GET, POST, PUT, DELETE, OPTIONS";
    set.headers["Access-Control-Allow-Headers"] = "Content-Type, Authorization";
    return new Response(null, { status: 204 });
  })
  .get("/", () => "Hello Elysia");

export type App = typeof app;
const port = process.env["PORT"] ?? 3000;

if (process.env.NODE_ENV !== "test") {
  app.listen(
    {
      port,
      hostname: "0.0.0.0",
    },
    () => {
      console.log(`🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`);
    },
  );
}
