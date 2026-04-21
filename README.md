# brasa (ember)

> A capability-native microkernel. Typed syscalls. BLAKE3 attestation from bootloader to process. The Nix store is the filesystem. The system is declared in tatara-lisp and the kernel converges.

**Brasa** — Brazilian-Portuguese for *ember, live coal*. The persistent fire of a running system — the spark that keeps userspace warm, the flame that never goes out as long as the machine is alive. Contrasts with its sibling `alicerce` (bedrock, passive, foundational): brasa is active, alive, burning.

**Status:** Phase 0 — Design. No code runs yet. This repo contains ADRs, architecture docs, and crate skeletons.

**License:** MIT. Fully open source from day one.

---

## What this is

`brasa` is a new operating system kernel written in Rust, authored in tatara-lisp, built with Nix, and rooted in the pleme-io typed-platform culture. It is explicitly **not** Linux-compatible at the ABI level. It is intentionally compatible with the pleme-io ecosystem: sui (Nix evaluator), tameshi (attestation), kensa (compliance), tatara-lisp (authoring), shikumi (config), and the broader Rust+Nix fleet.

**One-sentence pitch:** FluxCD for hardware — declare the system state, the kernel converges.

## What this is not

- Not a Linux distribution
- Not a Linux fork
- Not a Linux-compatibility layer
- Not a hobby kernel that imitates Unix
- Not trying to run shrink-wrapped software that wasn't compiled for it

## The stack

| Layer | Crate | Role |
|-------|-------|------|
| 6. Applications | (existing pleme-io apps, ported) | mado, tobirato, nexus services |
| 5. Runtime libraries | (existing pleme-io libs, ported) | garasu, egaku, madori |
| 4. Service framework | `floresta` (separate repo) | init + convergence loop, tatara-lisp authored |
| 3. Userspace drivers | `galho-*` (separate repos) | PCI, net, block, GPU — each driver its own crate |
| 2. Kernel interface | [`casca`](./crates/casca) | Typed syscall ABI |
| 1. Kernel core | [`tronco`](./crates/tronco) | Memory, scheduler, IPC primitives |
| 0. Boot + attestation | [`semente`](./crates/semente), [`raiz`](./crates/raiz), [`raizame`](./crates/raizame) | Bootloader, capability root, BLAKE3 chain |

Driver framework ([`galho`](./crates/galho)) and process abstraction ([`folha`](./crates/folha)) sit across the boundary, providing the types that kernel and userspace share.

## First target

**Apple Silicon laptop**, booting as a guest under the [`kasou`](https://github.com/pleme-io/kasou) hypervisor (Apple Virtualization.framework Rust binding).

- Arch: `aarch64`
- Drivers first wave: virtio-net, virtio-blk, virtio-console, virtio-9p, virtio-gpu
- Bare-metal Apple Silicon support is a later milestone; VM-first lets us iterate without writing custom Apple Silicon drivers up front.

`kasou` will grow `VZEFIBootLoader` support to host brasa guests. See [ADR-0006](./docs/adrs/0006-first-target-apple-silicon-kasou.md).

## Reading order

1. [docs/architecture.md](./docs/architecture.md) — the shape of the whole system
2. [docs/roadmap.md](./docs/roadmap.md) — phases 0-6 with milestone detail
3. [docs/naming.md](./docs/naming.md) — why the crates are named what they are
4. [docs/adrs/](./docs/adrs/) — design decisions, numbered

## Crates in this workspace

| Crate | Role |
|-------|------|
| [`semente`](./crates/semente) | Bootloader — measures `tronco` into attestation seed, hands off |
| [`tronco`](./crates/tronco) | Kernel core — memory, scheduler, IPC primitives (`no_std`, per-arch) |
| [`raiz`](./crates/raiz) | Capability system — typed caps, rights algebra, grant/revoke/derive |
| [`seiva`](./crates/seiva) | Typed IPC — message types, endpoints, async wait |
| [`casca`](./crates/casca) | Syscall ABI — the outer typed surface userspace touches |
| [`galho`](./crates/galho) | Driver framework — PCI/USB enumeration, device cap types |
| [`folha`](./crates/folha) | Process abstraction — the runtime shape of a userspace program |
| [`raizame`](./crates/raizame) | Attestation chain — BLAKE3 from bootloader to every process |

All eight are `no_std`-compatible. Userspace services and drivers live in separate repos.

## Related repos (separate, planned)

- **`floresta`** — userspace init + convergence orchestrator
- **`galho-virtio-net`**, **`galho-virtio-blk`**, **`galho-virtio-console`** — first wave of virtio drivers
- **`kasou`** — Apple Virtualization.framework Rust binding, will grow EFI boot support
- **`sui`** — Nix evaluator, target for kernel-adjacent integration
- **`tatara-lisp`** — authoring surface, target for `no_std` subset
- **`tameshi`**, **`sekiban`**, **`kensa`** — attestation infrastructure

## Build

Nothing builds yet. When it does:

```bash
nix develop                   # dev shell with aarch64 cross toolchain
cargo check --workspace       # typecheck
cargo test --workspace        # unit + proptest
nix build .#brasa-image       # bootable kasou guest image (Phase 1)
nix run .#brasa-qemu          # boot under QEMU (Phase 1)
nix run .#brasa-kasou         # boot under kasou (Phase 2)
```

## Contributing

Public repo, open ADR process. Any design change that affects the capability ABI or the attestation chain requires:

1. An ADR PR against [`docs/adrs/`](./docs/adrs/)
2. A proptest property asserting the invariant still holds
3. Updates to both kernel-side (`raiz` / `raizame`) and userspace-facing (`casca`) crates in lockstep

Syscall additions require explicit amendment to [ADR-0003](./docs/adrs/0003-syscall-surface.md) and may not push the total over the hard cap of 40 without a separate ADR debating the cap itself.

## License

MIT. See [LICENSE](./LICENSE).
