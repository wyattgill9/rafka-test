# Use the official Rust image as the base image with Alpine
FROM rust:stable-alpine AS builder

RUN apk add --no-cache \
    build-base \
    libssl-dev \
    ca-certificates

WORKDIR /usr/src/rafka

COPY . .


RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache \
    libssl1.1 \
    ca-certificates

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/rafka/target/release/rafka .

CMD ["./rafka"]
