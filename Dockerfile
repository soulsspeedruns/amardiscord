FROM rust:alpine AS build
ARG DATA_TARBALL_URL

RUN [ "$DATA_TARBALL_URL" != "" ] || exit 1
RUN apk add --no-cache build-base sqlite-dev curl
WORKDIR /build
COPY . .
RUN curl -LsSf "$DATA_TARBALL_URL" -o /build/amardiscord-data.tar.gz
RUN unzip /build/amardiscord-data.tar.gz -d /build
RUN cargo build --release --locked
RUN /build/target/release/amardiscord build

FROM alpine:latest

WORKDIR /app
COPY --from=build /build/target/release/amardiscord /app/amardiscord
COPY --from=build /build/amardiscord.sqlite /app/amardiscord.sqlite

ENTRYPOINT ["/app/amardiscord", "serve"]
