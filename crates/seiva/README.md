# seiva (sap) — typed IPC

Message types, endpoints, and the async wait primitive. The sap that carries
typed messages and capabilities between parts of the running tree.

See [`../../docs/architecture.md`](../../docs/architecture.md) and
[`ADR-0003`](../../docs/adrs/0003-syscall-surface.md).

## Phase 0 shape

- `Protocol` trait (sealed).
- Stubs for the Phase 1 protocols — virtio-console, virtio-net-device, virtio-blk-device — to be added alongside their respective driver crates.

## Constraints

- Messages are `repr(C)` POD types (zerocopy `FromBytes + IntoBytes`).
- Cap-passing messages are declared in the protocol; kernel enforces at send-time.
- No heap allocation in the send/receive path.
