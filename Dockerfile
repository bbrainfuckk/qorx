FROM rust:1-bookworm AS build

ARG QORX_VERSION=1.0.6
WORKDIR /src
COPY . .
RUN cargo build --release --locked \
    && test "$(./target/release/qorx --version)" = "qorx ${QORX_VERSION}"

FROM debian:bookworm-slim

ARG QORX_VERSION=1.0.6
LABEL org.opencontainers.image.title="Qorx" \
      org.opencontainers.image.version="${QORX_VERSION}" \
      org.opencontainers.image.source="https://github.com/bbrainfuckk/qorx" \
      org.opencontainers.image.licenses="AGPL-3.0-only"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /home/qorx --shell /usr/sbin/nologin qorx \
    && mkdir -p /data \
    && chown -R qorx:qorx /data

COPY --from=build /src/target/release/qorx /usr/local/bin/qorx

USER qorx
ENV QORX_HOME=/data
ENV QORX_BIND=0.0.0.0:47187

EXPOSE 47187
VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -fsS http://127.0.0.1:47187/health >/dev/null || exit 1

CMD ["qorx", "daemon"]
