FROM rust:alpine AS build
ARG DATA_ARCHIVE_URL

RUN [ "$DATA_ARCHIVE_URL" != "" ] || exit 1
RUN apk add --no-cache build-base sqlite-dev curl
WORKDIR /build
COPY . .

# Try extracting the archive both as a tarball and as a zip file.
# One of the two will work.
RUN curl -LsSf "$DATA_ARCHIVE_URL" -o /build/amardiscord-data
RUN unzip  /build/amardiscord-data -d /build/data || true
RUN tar xf /build/amardiscord-data -C /build/data || true

RUN cargo build --release --locked
RUN /build/target/release/amardiscord build || exit 1

FROM alpine:latest

WORKDIR /app
COPY --from=build /build/target/release/amardiscord /app/amardiscord
COPY --from=build /build/amardiscord.sqlite /app/amardiscord.sqlite

ENTRYPOINT ["/app/amardiscord", "serve"]
