FROM lukemathwalker/cargo-chef@sha256:c099c03e8a585e114631935ceb8bc7dbbfcfeed19e336feb305c9ef389030f72 AS chef
WORKDIR /app
RUN apt update && apt install libssl-dev pkg-config -y

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY crates/ crates/
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS rust-deps
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


FROM rust-deps AS rust-builder
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY crates/ crates/
RUN cargo build --release --bin backend --bin bot

#### Frontent build ####

FROM node:22-slim AS frontend-builder
WORKDIR /app
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ .
ARG VITE_API_BASE=""
ENV VITE_API_BASE=${VITE_API_BASE}
RUN npm run build


FROM debian:bookworm-slim AS app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /app/target/release/backend /usr/local/bin/backend
COPY --from=frontend-builder /app/dist /app/web/dist

WORKDIR /app

ENV BIND_ADDR="0.0.0.0:3001" \
    RUST_LOG="info"

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -sf http://${BIND_ADDR}/ -o /dev/null || exit 1

CMD ["backend"]

#### Bot build ####
FROM debian:bookworm-slim AS bot

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /app/target/release/bot /usr/local/bin/bot

ENV RUST_LOG="info"

CMD ["bot"]