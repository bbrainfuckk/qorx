FROM rust:1-bookworm AS build

WORKDIR /src
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim

LABEL org.opencontainers.image.source="https://github.com/bbrainfuckk/qorx"
LABEL org.opencontainers.image.licenses="AGPL-3.0-only"
LABEL org.opencontainers.image.description="Qorx Community Edition CLI"

COPY --from=build /src/target/release/qorx /usr/local/bin/qorx

ENTRYPOINT ["qorx"]
CMD ["--help"]
