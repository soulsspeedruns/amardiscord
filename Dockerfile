FROM rust:alpine AS build

RUN apk add --no-cache build-base sqlite-dev
WORKDIR /app

# Create blank project, so we can fetch dependencies in a separate layer before the actual build step
RUN cargo init --bin

COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release --locked

# Copy the rest of the source files, and build again
COPY src src
COPY templates templates
RUN cargo build --release --locked

FROM alpine:latest

WORKDIR /app
COPY --from=build /app/target/release/amardiscord /app/amardiscord

ENTRYPOINT ["/app/amardiscord", "./data"]
