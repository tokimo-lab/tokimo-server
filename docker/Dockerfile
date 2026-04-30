FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release -p tokimo-server -p tokimo-migration

FROM node:20-alpine AS admin-builder
WORKDIR /admin
COPY admin/ ./
RUN corepack enable && pnpm install --frozen-lockfile
RUN pnpm build

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tokimo-server /usr/local/bin/
COPY --from=builder /app/target/release/migration /usr/local/bin/
COPY --from=admin-builder /admin/dist /opt/tokimo-server/admin
WORKDIR /opt/tokimo-server
EXPOSE 5680
CMD ["tokimo-server"]
