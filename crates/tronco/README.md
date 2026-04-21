# tronco (trunk) — kernel core

The trunk of the tree. This crate is the brasa kernel core:

- Memory management (`mm`)
- Capability-based scheduler (`sched`)
- IPC dispatcher (`ipc`)
- Syscall implementations (`syscall`)
- Exception/interrupt handling (`trap`)
- Per-arch entry (`arch::aarch64`, `arch::x86_64`)

`no_std`, targets `aarch64-unknown-none` for kernel builds and `aarch64-apple-darwin` (with the `testing` feature) for host-side unit tests.

See [`../../docs/architecture.md`](../../docs/architecture.md) for the design, and [`ADR-0003`](../../docs/adrs/0003-syscall-surface.md) for the syscall surface tronco implements.

## Status

Phase 0 — Design. Module structure exists; all bodies are `todo!()` equivalents (empty modules with doc comments). Phase 1 milestone: boots under kasou via `VZEFIBootLoader`, prints a banner, handles one syscall.

## Constraints

- No allocations in the interrupt or page-fault path. Ever.
- No `unsafe` block longer than 20 lines. Each has a safety comment.
- No `#[allow(...)]` without an inline justification.
- `no_std` always. The `testing` feature enables `std` for host unit tests only.
