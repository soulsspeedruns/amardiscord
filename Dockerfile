FROM rust:alpine AS chef
WORKDIR /app

RUN apk add --no-cache build-base sqlite-dev
RUN cargo install cargo-chef 

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app

# Copy the recipe over to the build layer and build the dependencies only (will be cached)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy the rest of the source files, and build the actual app
COPY src src
COPY templates templates
RUN cargo build --release --locked

FROM alpine:latest
WORKDIR /app

COPY --from=builder /app/target/release/amardiscord amardiscord

ENTRYPOINT ["/app/amardiscord", "./data"]
