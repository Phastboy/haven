import { cors } from "@elysia/cors";
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

export const app = new Elysia({ prefix: "/api" })
  // CORS via the plugin. The hand-rolled version (a `.request` hook plus an
  // `options("/*")` catch-all) silently collapsed the whole app type to `any`,
  // which is what killed Eden's autocompletion in apps/web.
  .use(cors({ origin: process.env["WEB_ORIGIN"] ?? true, credentials: true }))
  .use(openapi())
  .use(configController)
  .use(createUserPlugin())
  .use(authController)
  .use(createOfferPlugin())
  .use(createOrderPlugin())
  .use(createFulfillmentPlugin())
  .get("/", () => "Hello Elysia");

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
