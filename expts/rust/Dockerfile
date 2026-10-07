# 1. Planner stage
FROM lukemathwalker/cargo-chef:latest-rust-1.99-bookworm AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 2. Builder stage
FROM lukemathwalker/cargo-chef:latest-rust-1.99-bookworm AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked -p haven-web

# 3. Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=builder /app/target/release/haven-web /app/haven-web
ENV HOST=0.0.0.0
ENV PORT=8000
EXPOSE 8000
ENTRYPOINT ["/app/haven-web"]
