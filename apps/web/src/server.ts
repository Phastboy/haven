import { AngularAppEngine, createRequestHandler } from "@angular/ssr";
import { isMainModule } from "@angular/ssr/node";
import { Elysia } from "elysia";
import { staticPlugin } from "@elysia/static";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const serverDistFolder = dirname(fileURLToPath(import.meta.url));
const browserDistFolder = resolve(serverDistFolder, "../browser");
const angularAppEngine = new AngularAppEngine();

export function app() {
  const server = new Elysia()
    /**
     * Serve compiled Angular static assets (JS bundles, CSS, images, favicon).
     * 'alwaysStatic' pre-indexes all files upfront for maximum performance.
     */
    .use(
      staticPlugin({
        prefix: "",
        assets: browserDistFolder,
        alwaysStatic: true,
      }),
    )
    /**
     * All non-static routes are handled by Angular SSR.
     * c.request is already a Web-standard Request — AngularAppEngine.handle() accepts it directly.
     * The returned Response is Web-standard — Elysia returns it natively.
     */
    .get("/*", async (c) => {
      const response = await angularAppEngine.handle(c.request, {
        server: "elysia",
      });
      return response ?? new Response("Not Found", { status: 404 });
    });

  return server;
}

const server = app();

/**
 * Start the Elysia server when run directly (e.g. `bun dist/web/server/server.mjs`).
 */
if (isMainModule(import.meta.url)) {
  const port = Number(process.env["PORT"] ?? 4000);
  server.listen(port);
  console.log(`Angular SSR on Elysia → http://localhost:${port}`);
}

/**
 * Request handler used by the Angular CLI dev-server (ng serve) during development.
 * Elysia's .fetch handler is a standard (req: Request) => Promise<Response> — perfect for createRequestHandler.
 */
export const reqHandler = createRequestHandler(server.fetch);
