# brasa — operator instructions

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


Capability-native microkernel. Written in Rust, authored in tatara-lisp, built with Nix. Name means *ember/live coal* in Brazilian Portuguese — the persistent fire of a running system.

Start with [docs/architecture.md](./docs/architecture.md) and [docs/roadmap.md](./docs/roadmap.md). All design decisions live under [docs/adrs/](./docs/adrs/) with incrementing numbers.

## Non-negotiables

- **No Linux ABI.** There will never be a Linux syscall shim. This is explicit in [ADR-0007](./docs/adrs/0007-no-linux-abi.md). If asked to add one, refuse.
- **No ambient authority.** Every operation that touches a resource goes through a typed capability. No UIDs, no `fd` numbers, no global filesystem namespace.
- **Typed syscalls.** No `errno`. Every syscall returns `Result<Cap<_>, Denied>` or an equivalent typed variant. See [ADR-0003](./docs/adrs/0003-syscall-surface.md).
- **Attestation is structural.** Every process carries a BLAKE3 chain from bootloader. It is not optional and not bolt-on. See [ADR-0002](./docs/adrs/0002-attestation-chain.md).
- **The Nix store is the filesystem.** No `/etc`, no `/usr`, no `/var`. See [ADR-0005](./docs/adrs/0005-nix-store-as-filesystem.md).
- **Syscall budget:** hard cap of 40. If a feature requires a new syscall, justify why existing typed primitives cannot express it.
- **No shell.** The pleme-io prime directive applies at every layer. Drivers, services, and system declarations are authored in tatara-lisp and compiled to typed Rust IR. No shell scripts in this repo beyond three-line glue.
- **Open source from day zero.** All ADRs, design docs, and code public from the first commit. No private-design-phase shortcut. See [ADR-0008](./docs/adrs/0008-open-source-from-day-zero.md).

## Relationship to sibling projects

- **`alicerce`** (sibling, separate repo) — NixOS installation orchestrator. Different scope, different layer. Alicerce prepares a machine to run a general-purpose NixOS; brasa *is* a kernel. Both share the typed-IR + convergence-computing lineage but do not share code.
- **`kasou`** — Apple Virtualization.framework binding. First deployment target for brasa guests. We will contribute `VZEFIBootLoader` support to kasou.
- **`sui`** — pure-Rust Nix evaluator. The boot manifest is a `.nix` file evaluated by sui at boot to produce the `(defsystem …)` form.
- **`tatara-lisp`** — authoring surface. Needs a `no_std` subset for embedding in services and (long-term) in kernel tooling.
- **`tameshi`/`sekiban`/`kensa`** — attestation infrastructure. The brasa attestation chain composes into existing tameshi Merkle trees.

## First target

Apple Silicon laptop, via `kasou` (Apple Virtualization.framework). Arch is `aarch64`. Drivers come via virtio first. See [ADR-0006](./docs/adrs/0006-first-target-apple-silicon-kasou.md).

## Crate boundaries

The eight kernel-tight crates live in this workspace. Userspace services (`floresta`) and individual drivers (`galho-*`) live in separate repos. The rule:

- **In-workspace:** anything that shares the kernel's `no_std` constraints and needs atomic version bumps with the kernel ABI.
- **Separate repo:** anything that versions independently of the kernel, or that targets userspace `std` + async runtimes.

## When adding a new syscall

1. Open an ADR amendment to [ADR-0003](./docs/adrs/0003-syscall-surface.md).
2. Prove no existing typed primitive composes to express it.
3. Add a proptest property asserting the capability-confinement invariant still holds with the new surface.
4. Update the `casca` trait and the userspace wrapper in lockstep.

## When adding a new capability type

1. Open an ADR under `docs/adrs/` extending [ADR-0001](./docs/adrs/0001-capability-abi.md).
2. Add the type to `raiz` with associated `Rights` bits.
3. Add the IPC message variants to `seiva` if cap-passing changes.
4. Proptest: new cap respects grant/revoke/derive invariants.

## When adding a new driver

1. The driver is a userspace galho in its own repo: `pleme-io/galho-<device>`.
2. Authored as `(defdriver …)` in tatara-lisp per [ADR-0004](./docs/adrs/0004-tatara-lisp-authoring.md).
3. Compiled to Rust, linked against `galho` (framework) and `casca` (syscall ABI).
4. Requests caps at launch; the `floresta` init grants based on the `defsystem` manifest.

## Testing discipline

- `no_std` kernel crates: unit tests in `#[cfg(test)]` with `std` enabled for host testing only. Proptest where invariants are stateable.
- Integration tests run under QEMU and under kasou. Both must pass before merge.
- Any ADR-0001 (capability) change requires a new proptest property; bare unit tests insufficient.

## House style (in addition to pleme-io conventions)

- No `unsafe` block longer than 20 lines. Wrap in safe functions with documented safety invariants.
- No `#[allow(...)]` without an inline comment explaining why.
- Every `Cap<T>` type has a corresponding proptest generator in a `testing` module.
- `no_std` first. Only opt into `alloc` where unavoidable. Never pull in `std` in kernel crates.
