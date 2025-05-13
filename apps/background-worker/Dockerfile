FROM rust:1.85.0-slim-bullseye AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release --bin background-worker

FROM debian:bullseye-slim

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/background-worker .

CMD ["./background-worker"]
