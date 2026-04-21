# folha (leaf) — process abstraction + userspace runtime

Leaves. The runtime shape of a userspace program. A Rust program that links against `folha::rt` gets a minimal startup routine, panic handler, and allocator — no libc.

## Phase 0 shape

- `CapBag` type — shape of the initial cap-bag delivered at spawn.
- `rt` module placeholder behind the `rt` feature gate.

## Phase 1 deliverables

- `brasa_main()` entry convention.
- Allocator backed by `MemCap`.
- Panic handler that emits a typed panic message on a `seiva::Endpoint`.
- `folha::rt::cap_bag()` — access to the initial cap bag.

## Why no libc

See [`../../docs/adrs/0007-no-linux-abi.md`](../../docs/adrs/0007-no-linux-abi.md). Libc's API is Linux-shaped at the bone; replacing it is part of the point.
