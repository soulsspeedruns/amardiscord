FROM rust:alpine AS build

RUN apk add --no-cache build-base sqlite-dev
WORKDIR /build
COPY . .

RUN cargo build --release --locked

FROM alpine:latest

RUN apk add --no-cache bash
WORKDIR /app
COPY serve.sh .
COPY --from=build /build/target/release/amardiscord /app/target/release/amardiscord

ENTRYPOINT ["/bin/bash", "serve.sh"]
