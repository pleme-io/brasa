# semente (seed) — bootloader

The seed. Loads the kernel, measures it into `H₀`, hands off.

Phase 0: lib-only. Phase 1: UEFI application (`aarch64-unknown-uefi`), booted under kasou via `VZEFIBootLoader`.

See:
- [`../../docs/adrs/0002-attestation-chain.md`](../../docs/adrs/0002-attestation-chain.md) — chain origin
- [`../../docs/adrs/0006-first-target-apple-silicon-kasou.md`](../../docs/adrs/0006-first-target-apple-silicon-kasou.md) — kasou hosting

## Handoff to tronco

`semente` produces a [`BootInfo`] struct containing:

- `tronco_hash`: BLAKE3 of the kernel ELF (the attestation seed).
- `memory_map`: physical memory layout from UEFI.
- `manifest`: pointer to the packed `BootManifest` (from `(defsystem …)`).

`tronco` reads `BootInfo` from `x0` (aarch64) at entry.

## Dependencies on kasou

Kasou needs `VZEFIBootLoader` support before semente can boot under it. Tracked as a separate PR against `pleme-io/kasou`.
