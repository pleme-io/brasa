# raiz (root) — capability system

The root. Every authority in brasa is a typed, unforgeable capability.

See [`../../docs/adrs/0001-capability-abi.md`](../../docs/adrs/0001-capability-abi.md) for the full design.

## Phase 0 shape

- `Cap<T, R>` type with compile-time rights.
- Stubs for Phase 1 cap types: `MemCap`, `VSpaceCap`, `CpuCap`, `IrqCap<N>`, `MmioCap`, `DmaCap`, `StorePathCap`.
- `Denied` enum matching ADR-0003.
- `testing` feature flag reserved for proptest invariant suites (Phase 1).

## Invariants (proven by proptest in Phase 1)

1. **Confinement** — cap flow is directional and explicit.
2. **Monotonic rights** — `derive` only reduces rights.
3. **Atomic revocation** — `revoke` invalidates all descendents.
4. **No forgery** — `CapId` is kernel-opaque.
5. **Parent-child direction** — children cannot grant upward.
