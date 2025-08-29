# Stage 1: build
FROM rust:1.79 as builder
WORKDIR /usr/src/app
COPY server/ .
RUN cargo build --release

# Stage 2: run
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/server /app/
ENV DATABASE_URL=postgres://postgres:postgres@expenses-db:5432/postgres
CMD ["./server"]
