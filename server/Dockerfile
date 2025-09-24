# Stage 1: build
FROM rust:1.89 AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

WORKDIR /usr/src/app/migration
RUN cargo build --release

# Stage 2: runtime
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libpq5 postgresql-client ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/server /app/server
COPY --from=builder /usr/src/app/migration/target/release/migration /app/migration

COPY static /app/static

ENV DATABASE_URL=postgres://postgres:postgres@expenses-database:5432/expenses

COPY scripts/docker-entrypoint.sh /app/docker-entrypoint.sh
RUN chmod +x /app/docker-entrypoint.sh

ENTRYPOINT ["/app/docker-entrypoint.sh"]
