FROM node:22.14.0-bookworm-slim AS frontend-builder
WORKDIR /workspace/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

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
# git daje odczyt stanu repozytorium przez GitRunner. Ten sam wzorzec: uruchamiane sa
# wylacznie polecenia czytajace, a worktree jest podmontowany read-only.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates curl git iproute2 network-manager \
    && rm -rf /var/lib/apt/lists/*
# Worktree nalezy do operatora hosta, a proces biegnie jako uid 10001. Bez tego git
# odmawia odczytu cudzego repozytorium ("dubious ownership") i widok gita jest pusty.
# Zgoda dotyczy wylacznie ODCZYTU - kontener nie ma zadnej sciezki zapisu do repozytorium.
RUN git config --system --add safe.directory '*'
RUN useradd --create-home --uid 10001 darkstar
COPY --from=builder /workspace/target/release/darkstar-server /usr/local/bin/darkstar-server
COPY --from=frontend-builder /workspace/frontend/dist /opt/darkstar/frontend-dist
ENV DARKSTAR_FRONTEND_DIST=/opt/darkstar/frontend-dist
USER darkstar
ENTRYPOINT ["/usr/local/bin/darkstar-server"]
