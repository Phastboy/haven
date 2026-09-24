import { cors } from "@elysia/cors";
import openapi, { fromTypes } from "@elysia/openapi";
import { Elysia } from "elysia";
import { createUserPlugin } from "./user/presentation/user.plugin";
import { createAuthPlugin } from "./auth";
import { createOfferPlugin } from "./offer/presentation/offer.plugin";
import { createDirectoryPlugin } from "./directory/presentation/graphql.plugin";
import { createOrderPlugin } from "./order/presentation/order.plugin";
import { createFulfillmentPlugin } from "./fulfillment/presentation/fulfillment.plugin";
import { createAutoCompletePlugin } from "./scheduler/auto-complete.plugin";

import { createMessagePlugin } from "./message/presentation/message.plugin";
import { createWsPlugin } from "./shared/presentation/ws.plugin";
import { NodeEventEmitterAdapter } from "./shared/infrastructure/node-event-bus.adapter";
import { SqlMessageRepository } from "./message/infrastructure/sql-message.repository";
import { CreateThreadUseCase } from "./message/application/create-thread.usecase";
import { DomainError } from "./shared/domain/errors";
import { SqlUserRepository } from "./user/infrastructure/sql-user.repository";

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

// Broadcast real-time notifications to users
eventBus.subscribe("order.requested", (payload) => {
  app.server?.publish(
    `user:${payload.ownerId}`,
    JSON.stringify({ type: "NOTIFICATION", data: { message: "Someone requested your offer!" } }),
  );
});

eventBus.subscribe("order.accepted", (payload) => {
  app.server?.publish(
    `user:${payload.requesterId}`,
    JSON.stringify({ type: "NOTIFICATION", data: { message: "Your request was accepted!" } }),
  );
});

eventBus.subscribe("message.created", (payload) => {
  app.server?.publish(
    `user:${payload.receiverId}`,
    JSON.stringify({ type: "NEW_MESSAGE", data: payload.message }),
  );
});

import { serverTiming } from "@elysia/server-timing";

const userRepo = new SqlUserRepository();

export const app = new Elysia({ prefix: "/api" })
  // CORS via the plugin. The hand-rolled version (a `.request` hook plus an
  // `options("/*")` catch-all) silently collapsed the whole app type to `any`,
  // `options("/*")` catch-all) silently collapsed the whole app type to `any`,
  // which is what killed Eden's autocompletion in apps/web.
  .use(cors({ origin: process.env["WEB_ORIGIN"] ?? true, credentials: true }))
  .use(serverTiming())
  .use(
    openapi({
      references: fromTypes(),
    }),
  )
  .use(createUserPlugin())
  .use(
    createAuthPlugin({
      async createProfileForAccount(accountId: string) {
        await userRepo.create({ accountId });
      },
    }),
  )
  .use(createOfferPlugin())
  .use(createOrderPlugin(eventBus))
  .use(createFulfillmentPlugin())
  .use(createMessagePlugin(eventBus))
  .use(createWsPlugin())
  .get("/", () => "Hello Elysia")
  .get("/health", () => ({ status: "ok", timestamp: new Date().toISOString() }))
  // Global last-resort error handler — registered LAST so plugin-level handlers
  // run first (Elysia resolves hooks in definition order within the same scope).
  // This ensures no raw SQL, stack traces, or Postgres internals ever reach the
  // client for errors that no plugin handler claimed.
  .error((context) => {
    const { error, set } = context;
    const asRecordContext = context as unknown as Record<string, unknown>;
    const asRecordError = error as unknown as Record<string, unknown>;
    const code = asRecordContext["code"] || asRecordError["code"];

    // 1. Custom Domain Errors
    if (error instanceof DomainError) {
      set.status = error.status;
      return {
        type: error.type,
        title: error.title,
        status: error.status,
        detail: error.detail,
      };
    }

    // 2. Validation Errors (TypeBox/Elysia)
    if (code === "VALIDATION") {
      set.status = 422;
      return {
        type: "validation_error",
        title: "Validation Failed",
        status: 422,
        detail: "The request payload failed to validate against the schema.",
        errors: asRecordError["all"] ?? error.message,
      };
    }

    // 3. Not Found Errors (Elysia unmatched routes)
    const asAny = error as unknown as Record<string, unknown>;
    if (
      code === "NOT_FOUND" ||
      asAny["code"] === "not-found" ||
      (error instanceof Error &&
        "status" in error &&
        (error as unknown as { status: number }).status === 404)
    ) {
      set.status = 404;
      return {
        type: "not_found",
        title: "Resource Not Found",
        status: 404,
        detail: "The requested route or resource does not exist.",
      };
    }

    // 4. Unhandled Internal Errors
    console.error("[unhandled error]", error);
    set.status = 500;
    return {
      type: "internal_error",
      title: "Internal Server Error",
      status: 500,
      detail: "An unexpected error occurred.",
    };
  });

export type App = typeof app;

// Registered after `App` is captured: both erase the inferred route map
// (croner's `Cron` type is not nameable; the GraphQL plugin returns `any`),
// and that erasure propagates to `treaty<App>` in apps/web.
app.use(createDirectoryPlugin());
app.use(createAutoCompletePlugin());
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
