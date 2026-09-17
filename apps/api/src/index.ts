import openapi from "@elysia/openapi";
import { Elysia } from "elysia";
import { createUserPlugin } from "./user/presentation/user.plugin";
import { authController } from "./auth";

import { configController } from "./config.controller";

const app = new Elysia({ prefix: '/api' })
  .use(openapi())
  .use(configController)
  .use(createUserPlugin())
  .use(authController)
  .request(({ set }) => {
    set.headers["Access-Control-Allow-Origin"] = "*";
    set.headers["Access-Control-Allow-Methods"] = "GET, POST, PUT, DELETE, OPTIONS";
    set.headers["Access-Control-Allow-Headers"] = "Content-Type, Authorization";
  })
  .options("/*", ({ set }) => {
    set.headers["Access-Control-Allow-Origin"] = "*";
    set.headers["Access-Control-Allow-Methods"] = "GET, POST, PUT, DELETE, OPTIONS";
    set.headers["Access-Control-Allow-Headers"] = "Content-Type, Authorization";
    return new Response(null, { status: 204 });
  })
  .get("/", () => "Hello Elysia");

export type App = typeof app;

app.listen(
    {
      port: 3000,
      hostname: "0.0.0.0",
    },
    () => {
      console.log(
        `🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`,
      );
    },
  );
