FROM rust:alpine AS build

RUN apk add --no-cache build-base sqlite-dev
WORKDIR /build
COPY . .
RUN cargo build --release --locked
RUN /build/target/release/amardiscord build

FROM alpine:latest

WORKDIR /app
COPY --from=build /build/target/release/amardiscord /app/amardiscord
COPY --from=build /build/amardiscord.sqlite /app/amardiscord.sqlite

ENTRYPOINT ["/app/amardiscord", "serve"]
