# brasa — architecture

> This document is the canonical shape of the system. When it disagrees with code, fix the code or update the document and open an ADR explaining why.

## Top-level picture

```
┌──────────────────────────────────────────────────────────────┐
│  Layer 6: Applications        │ mado, tobirato, nexus services │
│  Layer 5: Runtime libraries   │ garasu, egaku, madori (ported) │
│  Layer 4: Service framework   │ floresta (init) + tatara-lisp  │
│  Layer 3: Userspace drivers   │ galho-* (virtio-net, blk, …)    │
│  ─────────────── userspace ───────────────────────────────────  │
│  Layer 2: Kernel interface    │ casca (typed syscall surface)  │
│  Layer 1: Kernel core         │ tronco (MM, sched, IPC)        │
│  Layer 0: Boot + attestation  │ semente → raiz (cap root)      │
└──────────────────────────────────────────────────────────────┘
```

Everything at Layer 3 and above runs in userspace. The kernel has **no** drivers, **no** filesystems, **no** network stack — just memory management, scheduling, IPC, capabilities, and arch primitives.

## The five load-bearing design choices

These are the decisions that make brasa brasa. Everything else follows from them.

### 1. Capabilities instead of ambient authority

No process has any authority except what it holds a typed capability for. `Cap<T, R>` is the primitive — unforgeable, revocable, non-`Copy`. Every syscall takes caps as arguments and returns caps (or typed errors). No UIDs. No `fd` numbers. No global filesystem namespace. See [ADR-0001](./adrs/0001-capability-abi.md).

### 2. Attestation is a kernel primitive

Every process carries a BLAKE3 chain rooted at the bootloader. `semente` measures `tronco`; `tronco` measures `floresta`; `floresta` measures each service it spawns. At any point a process can ask the kernel for its attestation chain and receive a `Vec<Hash>` verifiable against `tameshi`-signed release artifacts. This is not a daemon; it is structural. See [ADR-0002](./adrs/0002-attestation-chain.md).

### 3. Typed syscalls, no errno

Syscalls return `Result<Cap<T, R>, Denied>` or a typed variant. There is no integer `errno`. There is no `-1` sentinel. The `casca` crate is the single source of truth for the ABI and is shared between kernel and userspace. See [ADR-0003](./adrs/0003-syscall-surface.md).

### 4. tatara-lisp is the authoring surface

Drivers, services, capability types, and whole-system manifests are authored as `(def…)` forms in tatara-lisp and compiled to typed Rust IR by `forja`. The kernel does not parse Lisp at runtime; it loads a pre-compiled typed manifest produced at build time. See [ADR-0004](./adrs/0004-tatara-lisp-authoring.md).

### 5. The Nix store is the filesystem

There is no `/etc`, `/usr`, `/var`, or `/home`. The only durable namespace is `/nix/store/`. Mutable state is served via typed `StorePathCap`s or `MutableBlobCap`s — not via an ambient writable directory. Configuration reaches services via caps, not via files the service reads by path. See [ADR-0005](./adrs/0005-nix-store-as-filesystem.md).

## Crate-by-crate shape

Each crate below is a member of the workspace at `crates/<name>/`. All are `no_std` unless explicitly marked. All target `aarch64-unknown-none` for the kernel-side build and `aarch64-unknown-none` or `aarch64-apple-darwin` for host tests.

### `semente` — the seed

The bootloader. Entry point for EFI firmware (eventually `VZEFIBootLoader` under kasou). Responsibilities:

- Load the kernel ELF (`tronco.elf`) into memory.
- Measure it: compute `H₀ = BLAKE3(tronco.elf)`.
- Set up the initial page tables, interrupt vector table, serial console.
- Hand off to `tronco` with `H₀` and a typed boot-info struct in known registers.

Phase 0 state: skeleton. Phase 1: boots under kasou, produces a measurement, jumps to a `tronco` stub that prints a typed message.

### `tronco` — the trunk (kernel core)

The kernel itself. `no_std`, per-arch. Modules:

- `arch::{aarch64, x86_64}` — CPU primitives, MMU setup, exception vectors, timer.
- `mm` — physical frame allocator, virtual address spaces, page tables.
- `sched` — capability-based scheduler; no PIDs, processes are addressed by `VSpaceCap`.
- `ipc` — kernel-side implementation of typed message passing; see `seiva` for the type machinery.
- `syscall` — the dispatch table; delegates to `casca`-defined handlers.
- `trap` — exception/interrupt handling, syscall entry, panic handling.

Tronco never allocates in the page-fault or interrupt path. All kernel allocation is bump-style from pre-reserved regions, with long-lived structures held in slab allocators.

### `raiz` — the root (capability system)

The capability table and rights algebra.

```rust
pub struct Cap<T: CapType, R: Rights = Full> {
    handle: CapId,          // opaque kernel-table index
    _t: PhantomData<(T, R)>,
}

impl<T: CapType, R: Rights> !Copy for Cap<T, R> {}
```

Operations: `grant`, `revoke`, `derive` (narrow rights), `pass` (via IPC). Every `Cap<T>` must have a `proptest` generator in a `testing` module. The core invariant — *if process P does not hold a cap to object O, no sequence of syscalls gets P a cap to O* — is proven by proptest on every PR that touches `raiz`.

Cap families in scope for Phase 1:

| Type | Represents |
|------|------------|
| `MemCap` | A physical memory region |
| `VSpaceCap` | A virtual address space (what "a process" is, at the type level) |
| `CpuCap` | CPU time budget + scheduling class |
| `IrqCap<N>` | A specific IRQ line |
| `MmioCap` | A specific MMIO range (device-bound) |
| `DmaCap` | DMA-safe memory + IOMMU ticket |
| `EndpointCap<P>` | A typed IPC endpoint for protocol `P` |
| `StorePathCap` | A specific Nix store path, read-only mmap |
| `ServiceCap<S>` | A running service handle |

### `seiva` — the sap (typed IPC)

Message types, endpoints, async wait. `EndpointCap<Protocol>` means the endpoint carries a typed protocol; the type system enforces which messages you can send and receive. Cap-passing is encoded in the message type (a message containing a `Cap<T>` transfers ownership on send).

Seiva is used by both kernel (for implementing IPC) and userspace (for speaking to services). It must compile under `no_std`.

### `casca` — the bark (syscall ABI)

The outer typed surface userspace touches. A single trait, `Casca`, declares the full syscall set. Kernel and userspace both consume it — the kernel implements the trait, userspace has a `casca::userspace` module that issues the `svc` instruction and marshals args.

Syscall budget: **hard cap of 40.** Phase 1 target is ~25 syscalls total. If you want to add a new one, it requires an ADR-0003 amendment and a proof that no existing primitive composes to express it.

### `galho` — the branch (driver framework)

Types that drivers share:

- PCI/USB enumeration primitives.
- `DeviceCap<Dev>` — a cap narrowed to a specific device.
- `IrqCap<N>`, `MmioCap`, `DmaCap` integrations.
- Driver lifecycle traits: `init`, `attach`, `detach`, `probe`.

Drivers themselves live in separate repos (`pleme-io/galho-virtio-net`, etc.) and depend on this crate + `casca` + `seiva`.

### `folha` — the leaf (process abstraction)

The runtime shape of a userspace program: entry point ABI, TLS layout, signal-free exception model, cap bag format. Shared between the kernel (which spawns processes) and userspace startup code (the `folha::rt` runtime that a program links against).

`folha::rt` replaces libc. Rust programs link against it instead of `std`; they get a minimal panic handler, allocator, and a startup routine that extracts the initial cap bag from the spawn message.

### `raizame` — the root-chain (attestation chain)

The BLAKE3 chain extension. Integrates with `tameshi` (existing) to verify signatures on each link.

```
Hₙ = BLAKE3(Hₙ₋₁ ∥ image_hash ∥ initial_caps_sig)
```

`raizame` provides the kernel syscall (`cap_attest_chain()`) and the userspace reading API. It does not *verify* signatures — that is `kensa`'s job — but it produces chains that can be verified.

## The build pipeline

```
tatara-lisp source (defsystem, defservice, defdriver, defcapability)
    │
    ▼ forja (existing tatara compiler)
typed Rust IR
    │
    ▼ rustc + crate2nix
brasa kernel image (ELF + initial cap bag + attested manifest)
    │
    ▼ semente (bootloader packaging)
bootable disk image (EFI)
    │
    ▼ kasou (Phase 2) / QEMU (Phase 1) / bare metal (Phase 3+)
running kernel with declared services
```

A pleme-io developer authors their system in tatara-lisp; runs `nix build .#brasa-image`; receives a bootable artifact with a BLAKE3 hash anchored to the tameshi release signing key. No raw Rust for drivers — that is a generated artifact from the `(defdriver …)` form.

## What lives where

| Concern | Location |
|---------|----------|
| Capability types | `raiz` |
| Syscall declarations | `casca` |
| Kernel syscall implementations | `tronco::syscall` |
| Userspace syscall issue | `casca::userspace` (userspace-only module) |
| IPC message type machinery | `seiva` |
| Kernel-side IPC dispatcher | `tronco::ipc` |
| Bootloader | `semente` |
| Attestation chain machinery | `raizame` |
| Attestation verification | `tameshi`/`kensa` (separate repos) |
| Init / service orchestration | `floresta` (separate repo) |
| Individual drivers | `galho-<device>` (separate repos) |
| Driver framework types | `galho` |
| Process runtime (no-libc) | `folha::rt` |

## Cross-cutting invariants

These hold at every checkpoint and are enforced by proptest where possible:

1. **Cap confinement:** ∀P, O. P cannot reach a cap for O without explicit grant.
2. **Monotonic rights:** Cap derivation can only *reduce* rights. Never elevate.
3. **Attestation totality:** Every running process has a computable chain back to `semente`.
4. **Syscall budget:** |syscalls| ≤ 40.
5. **Unsafe blocks:** no block exceeds 20 lines; each has a safety comment.
6. **No `std` in kernel crates:** checked by `#![no_std]` + workspace lints.

## Open architectural questions

Tracked in [roadmap.md](./roadmap.md) under "Open questions before Phase 1." Summary:

- IOMMU requirement hard or soft?
- `sui` in-kernel or as a userspace service?
- Per-core or global run queue?
- Which GPU first (bare-metal only matters for Phase 5)?
- Verification ambition: proptest-only or Kani/Creusot?

See roadmap for the decision criteria on each.
