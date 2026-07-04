FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY ./ ./

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/libretto /usr/local/bin/libretto
COPY --from=builder /app/migrations /data/migrations

RUN mkdir -p /data
WORKDIR /data

CMD ["libretto", "api"]
