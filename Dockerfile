FROM rust:1-bookworm AS builder
WORKDIR /workspace
COPY . .
RUN cargo test --workspace --locked
RUN cargo build --release -p darkstar-server

FROM debian:bookworm-slim
# iproute2 daje `ip`, network-manager daje `nmcli`. Oba sluza WYLACZNIE do odczytu
# stanu bramy przez GatewayProvider. Przy network_mode host kontener dzieli stos
# sieciowy z maszyna, wiec `ip` widzi te same interfejsy; `nmcli` rozmawia z
# NetworkManagerem hosta przez gniazdo DBus podmontowane read-only.
# CommandRunner nie ma metody zapisu, wiec zadne z tych narzedzi nic nie zmieni.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl iproute2 network-manager \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --create-home --uid 10001 darkstar
COPY --from=builder /workspace/target/release/darkstar-server /usr/local/bin/darkstar-server
USER darkstar
ENTRYPOINT ["/usr/local/bin/darkstar-server"]
