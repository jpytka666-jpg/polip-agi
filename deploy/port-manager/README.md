# Dark Star Port Manager

Central registry and operator for host-port allocation. Container ports remain application contracts; host ports are allocated from the registered pool and checked against live listening sockets.

## Commands

```bash
./deploy/port-manager/port-manager.sh allocate darkstar
./deploy/port-manager/port-manager.sh list
./deploy/port-manager/port-manager.sh audit
./deploy/port-manager/port-manager.sh release darkstar
```

Allocated runtime state is intentionally local and ignored by Git. The tracked registry defines the pool and service contracts; the generated `deploy/.env` supplies the current host port to Compose.
