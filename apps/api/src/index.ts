import { config } from "./config";
import { createDb } from "./database/db";
import { cors } from "@elysia/cors";
import openapi, { fromTypes } from "@elysia/openapi";
import { Elysia } from "elysia";
import { serverTiming } from "@elysia/server-timing";

import { createUserPlugin } from "./user/presentation/user.plugin";
import { createAuthPlugin } from "./auth";
import { createOfferPlugin } from "./offer/presentation/offer.plugin";
import { createDirectoryPlugin } from "./directory/presentation/graphql.plugin";
import { createOrderPlugin } from "./order/presentation/order.plugin";
import { createFulfillmentPlugin } from "./fulfillment/presentation/fulfillment.plugin";
import { createAutoCompletePlugin } from "./scheduler/auto-complete.plugin";
import { createMessagePlugin } from "./message/presentation/message.plugin";
import { createWsPlugin } from "./shared/presentation/ws.plugin";

import { PostgresEventBusAdapter } from "./shared/infrastructure/postgres-event-bus.adapter";
import { SqlUserRepository } from "./user/infrastructure/sql-user.repository";
import { setupEventSubscribers } from "./shared/infrastructure/event-subscribers";
import { globalErrorHandler } from "./shared/presentation/error-handler.plugin";

const eventBus = new PostgresEventBusAdapter(config.DATABASE_URL);

const db = createDb(config);
const userRepo = new SqlUserRepository(db);

const v1 = new Elysia()
  .use(createUserPlugin(db))
  .use(
    createAuthPlugin(
      {
        async createProfileForAccount(accountId: string) {
          await userRepo.create({ accountId });
        },
      },
      config,
      db
    ),
  )
  .use(createOfferPlugin(db));

const v2 = new Elysia()
  .use(createOrderPlugin(eventBus, db))
  .use(createFulfillmentPlugin(db))
  .use(createMessagePlugin(eventBus, db))
  .use(createWsPlugin(db));

export const app = new Elysia({ prefix: "/api" })
  .use(cors({ origin: config.WEB_ORIGIN, credentials: true }))
  .use(serverTiming())
  .error(globalErrorHandler)
  .use(
    openapi({
      references: fromTypes(),
    }),
  )
  .use(v1)
  .use(v2)
  .get("/", () => "Hello Elysia")
  .get("/health", () => ({ status: "ok", timestamp: new Date().toISOString() }));

// Fix the event subscribers to actually have the app reference for WS publishing
await setupEventSubscribers(eventBus, app, db);

export type App = typeof app;

// Registered after `App` is captured
app.use(createDirectoryPlugin(db));
app.use(createAutoCompletePlugin(db));

if (config.NODE_ENV !== "test") {
  app.listen(
    {
      port: config.PORT,
      hostname: config.HOST,
    },
    () => {
      console.log(`🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`);
    },
  );
}
