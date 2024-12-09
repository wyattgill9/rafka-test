# Use the official Rust image with Alpine (use rust:alpine or a specific version like rust:1.82-alpine)
FROM rust:alpine as builder

# Install necessary dependencies for building the Rust project
RUN apk add --no-cache \
    build-base \   
    libssl3 \
    ca-certificates

# Set the working directory inside the container
WORKDIR /usr/src/rafka

# Copy the source code into the container
COPY . .

# Build the application
RUN cargo build --release

# Create a smaller final image based on Alpine
FROM alpine:latest

# Install necessary runtime dependencies (using libssl3 instead of libssl1.1)
RUN apk add --no-cache \
    libssl3 \
    ca-certificates

# Set the working directory
WORKDIR /usr/local/bin

# Copy the built executable from the builder image
COPY --from=builder /usr/src/rafka/target/release/rafka .

# Run the application
CMD ["./rafka"]
