FROM node:22-slim AS web
WORKDIR /app/web
RUN corepack enable
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY web ./
RUN pnpm build

FROM rust:1 AS server
WORKDIR /app
COPY . .
RUN cargo build --release -p server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates git && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=server /app/target/release/server /app/server
COPY --from=web /app/web/dist /app/web/dist
ENV WEB_DIST=/app/web/dist BIND_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["/app/server"]
