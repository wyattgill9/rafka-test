# Rafka Dockerfile
FROM rust:1.82-alpine as builder

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/rafka

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest

RUN apk add --no-cache \
    libssl3 \
    ca-certificates

COPY --from=builder /usr/src/rafka/target/x86_64-unknown-linux-musl/release/rafka /usr/local/bin/rafka

EXPOSE 8080

CMD ["/usr/local/bin/rafka"]