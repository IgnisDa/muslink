FROM rust:1.85.0-slim-bullseye AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/telegram-bot .

CMD ["./telegram-bot"]
