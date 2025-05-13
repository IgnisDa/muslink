FROM rust:1.85.0-slim-bullseye AS builder

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release --bin graphql-api

FROM debian:bullseye-slim

WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/graphql-api .

CMD ["./graphql-api"]
