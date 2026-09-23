import { aot } from "elysia/plugin/aot/bun";

await Bun.build({
  entrypoints: ["src/index.ts"],
  compile: { outfile: "./apibin" },
  plugins: [aot("src/index.ts")],
});

process.exit(0);
