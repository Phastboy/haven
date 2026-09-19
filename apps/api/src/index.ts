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
  .use(openapi())
  .use(configController)
  .use(createUserPlugin())
  .use(authController)
  .use(createOfferPlugin())
  .use(createDirectoryPlugin())
  .use(createOrderPlugin())
  .use(createFulfillmentPlugin())
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
