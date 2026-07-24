FROM rust:1.85-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --locked --release

FROM docker:29.1.5-cli AS docker-cli

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl git \
    && rm -rf /var/lib/apt/lists/*

COPY --from=docker-cli /usr/local/bin/docker /usr/local/bin/docker
COPY --from=docker-cli /usr/local/libexec/docker/cli-plugins /usr/local/libexec/docker/cli-plugins
COPY --from=builder /build/target/release/izyploy /usr/local/bin/izyploy

ENV DATABASE_URL=sqlite:///data/izyploy.db \
    WORKSPACE_ROOT=/data/workspaces \
    BIND_ADDRESS=0.0.0.0:3000 \
    RUNTIME_HOST=host.docker.internal

VOLUME ["/data"]
EXPOSE 3000

ENTRYPOINT ["izyploy"]
