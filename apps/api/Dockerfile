FROM oven/bun:1.4.2 AS builder
WORKDIR /app

# Copy root monorepo files needed for dependencies
COPY package.json bun.lock ./
COPY apps/api/package.json ./apps/api/
COPY apps/web/package.json ./apps/web/

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code and build
COPY . .
WORKDIR /app/apps/api
RUN bun run build.ts

# Production minimal runtime
FROM debian:bookworm-slim AS runner
WORKDIR /app

# Install certificates and curl (for docker healthcheck)
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Create system user and copy compiled binary
RUN useradd -r -s /bin/false haven
COPY --from=builder --chown=haven:haven /app/apps/api/apibin ./haven-api

USER haven

EXPOSE 3000
CMD ["./haven-api"]
