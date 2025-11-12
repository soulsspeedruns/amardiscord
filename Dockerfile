FROM rust:alpine AS build

RUN apk add --no-cache build-base sqlite-dev
WORKDIR /build
COPY . .

RUN cargo build --release --locked

FROM alpine:latest

WORKDIR /app
COPY --from=build /build/target/release/amardiscord /app/amardiscord

ENTRYPOINT ["/app/amardiscord", "/app/data"]
