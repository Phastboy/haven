FROM oven/bun:1.4.2 AS builder
WORKDIR /app

# Copy dependencies
COPY package.json bun.lock ./
COPY apps/api/package.json ./apps/api/
COPY apps/web/package.json ./apps/web/

RUN bun install --frozen-lockfile

# Copy source and build
COPY . .
WORKDIR /app/apps/web
RUN bun run build

# Runner stage
FROM oven/bun:1.4.2-slim AS runner
WORKDIR /app

# Copy necessary files for Astro Bun SSR
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/apps/web/node_modules ./apps/web/node_modules
COPY --from=builder /app/apps/web/dist ./apps/web/dist
COPY --from=builder /app/apps/web/package.json ./apps/web/package.json

RUN chown -R bun:bun /app

WORKDIR /app/apps/web
ENV HOST=0.0.0.0
ENV PORT=4200
USER bun
EXPOSE 4200
CMD ["bun", "run", "./dist/server/entry.mjs"]
