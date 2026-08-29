FROM rust:1-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo test --workspace --locked
RUN cargo build --release -p darkstar-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --create-home --uid 10001 darkstar
COPY --from=builder /workspace/target/release/darkstar-server /usr/local/bin/darkstar-server
USER darkstar
ENTRYPOINT ["/usr/local/bin/darkstar-server"]
