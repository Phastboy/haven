import openapi from "@elysia/openapi";
import { Elysia } from "elysia";
import { createUserPlugin } from "./user/presentation/user.plugin";

const app = new Elysia()
  .use(openapi())
  .use(createUserPlugin())
  .get("/", () => "Hello Elysia")
  .listen(
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

