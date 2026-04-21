# brasa — status

> **Stopping point: 2026-04-21, brasa@120bf16**
> Phase 0 exit criterion proven; durable pause before Phase 1 implementation work.

This file is the single source of truth for *what is demonstrably true today*. If it says "green", there is a reproducible command a newcomer can run that produces the green outcome; if it says "pending", the work is not yet done or not yet reproducibly verified.

When anything changes, update this file in the same commit.

## The proven state

### M0 — boot (green ✓)

`brasa-bin` Phase 0 image builds for `aarch64-unknown-none`, boots under QEMU `virt`, writes a typed banner to the PL011 UART at `0x0900_0000`, halts cleanly on `wfe`.

Reproducible command (from the brasa repo root):

```bash
nix develop --command bash -c '
  cargo build -p brasa-bin --target aarch64-unknown-none --release
  timeout 3 qemu-system-aarch64 -machine virt -cpu cortex-a72 -m 128 \
    -display none -serial file:./.serial-probe.log \
    -kernel target/aarch64-unknown-none/release/brasa-kernel < /dev/null
  cat ./.serial-probe.log
'
```

Expected output (266 bytes):

```

======================================
 brasa — ember kindled
======================================
 phase:  Phase 0 — Design
 arch:   aarch64-unknown-none
 host:   QEMU virt
 target: <nothing yet — halting>
======================================

```

## Repos in play

| Repo | HEAD | Phase | Notes |
|---|---|---|---|
| [`pleme-io/brasa`](https://github.com/pleme-io/brasa) | `120bf16` | 0 (proven) | Kernel workspace. 9 crates. Boots under QEMU. |
| [`pleme-io/floresta`](https://github.com/pleme-io/floresta) | `main` | 0 (design) | Userspace init. Scaffolded; no binary yet. |
| [`pleme-io/galho-virtio-console`](https://github.com/pleme-io/galho-virtio-console) | `main` | 0 (design) | First driver. Scaffolded; no implementation. |
| [`pleme-io/galho-virtio-net`](https://github.com/pleme-io/galho-virtio-net) | `main` | 0 (design) | L2 net driver. Scaffolded. |
| [`pleme-io/galho-virtio-blk`](https://github.com/pleme-io/galho-virtio-blk) | `main` | 0 (design) | Raw block driver. Scaffolded. |
| [`pleme-io/kasou`](https://github.com/pleme-io/kasou) | `faa5371` | 1-ready | `VZEFIBootLoader` support landed. 46 tests green. |
| [`pleme-io/kikai`](https://github.com/pleme-io/kikai) | `e27d8d8` | (out-of-project) | Call-site follow-up for `kasou::BootConfig::Linux`. |

Pending repos (planned, not yet created):

- `pleme-io/jabuti-store` — Nix-store-as-capability service. Phase 2 blocker.
- `pleme-io/wasmtime-on-brasa` — WASM/WASI runtime service. Phase 3-4 (ADR-0009).
- `pleme-io/galho-virtio-9p` — shared-directory driver. Phase 2 convenience.
- `pleme-io/brasa-forja-ext` — tatara-lisp domain registrations for brasa forms (`defdriver`, `defservice`, …). Phase 1.

## Green checklist

- [x] `nix develop` on brasa resolves the dev shell (fenix + qemu + llvm + cargo) without manual sha256.
- [x] `cargo build --workspace` succeeds for host target (stable Rust 1.90+).
- [x] `cargo build -p brasa-bin --target aarch64-unknown-none --release` produces a valid AArch64 EXEC ELF with entry `0x4008_0000`.
- [x] QEMU virt boot produces the expected 266-byte serial banner.
- [x] `cargo test` in `kasou` passes (46 tests).
- [x] `cargo check` in `kikai` passes against the new `BootConfig::Linux` variant.
- [x] All eight ADRs merged and indexed.
- [x] Vision, roadmap, naming, status, tracking docs cross-link cleanly.
- [x] ADR-0009 (WASM/WASI first-class) accepted.

## Known caveats

- **`sha256 = lib.fakeSha256`** placeholders in `floresta` / `galho-virtio-console` / `galho-virtio-net` / `galho-virtio-blk` flakes. `brasa` is fixed (uses `fenix.combine`). The rest follow the same fix when they grow real build targets.
- **Unused-import warning cleared** in `casca` (re-exported). No lint warnings in the workspace at Phase 0 HEAD.
- **`casca::Casca` trait body is empty** — Phase 0 only reserves the trait. Phase 1 populates the 25 Phase-1 syscall signatures per [ADR-0003](./adrs/0003-syscall-surface.md).
- **No proptest has run yet.** `raiz::testing::invariants` is declared, not implemented. Phase 1 M3 is the first real test.
- **Attestation chain is declared, not produced.** `raizame::ProcessChain::extend` works on the host (logic-only); it is not yet called from any boot path. Phase 1 M2/M4.
- **No real devices.** The only PoC runs against QEMU virt's synthetic hardware. Kasou guest integration requires ADR-0006's `VZEFIBootLoader` Milestone (in kasou, done) + a proper EFI packaging step for `brasa-bin` (not yet).

## Session highlights — 2026-04-20 → 2026-04-21

What got done in the founding session:

- Vision agreed, name agreed (`brasa`), ADR spine drafted and merged (8 ADRs at first commit, 9 at HEAD).
- Workspace structure established. Eight kernel-tight crates scaffolded with `Cargo.toml` + `src/lib.rs` + per-crate `README.md`.
- Four sibling repos created and pushed: `floresta`, `galho-virtio-{console,net,blk}`.
- `kasou` extended with `VZEFIBootLoader` + `VZEFIVariableStore` support, maintaining API compatibility with downstream consumers (`kikai` follow-up committed).
- `brasa-bin` Phase 0 PoC built and **actually booted** under QEMU with a clean 266-byte UART banner.
- Toolchain dialed from speculative-nightly to stable Rust 1.82+. No nightly needed.
- WASM/WASI accepted as a first-class runtime via ADR-0009 — the route to ecosystem compatibility without an ABI shim.
- Comprehensive docs: [vision.md](./vision.md), [roadmap.md](./roadmap.md), [naming.md](./naming.md), [status.md](./status.md) (this), [tracking.md](./tracking.md).

## How to pick up cold

If you're coming back to this project with no context:

1. Read [`README.md`](../README.md) for the pitch and layer map.
2. Read [`docs/vision.md`](./vision.md) for the thesis (safety + mutability compose via the rust/lisp line; WASM/WASI bridges the ecosystem).
3. Skim [`docs/adrs/README.md`](./adrs/README.md) for the decision spine.
4. Run the reproducible boot command above to validate the environment is working.
5. Open [`docs/tracking.md`](./tracking.md) to see the next unchecked box.
6. Pick one of the four "immediate moves" below.

## Immediate next moves (session 2)

In order of recommended priority:

1. **M1 — first syscall.** Flesh out the first two `casca::Casca` trait methods (`cap_attest_chain`, `time_now`). Implement kernel-side in `tronco::syscall`. Extend `brasa-bin` to make one syscall from userspace and print the returned chain view. Proves the rust/lisp line reaches across the kernel boundary.
2. **M3 — first proptest.** Build a testable `CapTable` model in `raiz::testing::model` (std-gated). Implement grant/revoke/derive. Land the confinement property over 10,000 proptest cases. Proves the safety thesis is machine-checkable.
3. **M2 — first Lisp-authored driver.** Stand up `pleme-io/brasa-forja-ext` with the `defdriver` domain registration. Express `galho-virtio-console` as a `(defdriver virtio-console …)` form. Round-trip through `forja` produces the Rust we already have. Proves the authoring surface closes end-to-end.
4. **Kasou ↔ brasa boot.** Package `brasa-bin` as an EFI PE. Add a kasou test that boots it via `VZEFIBootLoader`. Proves the real Phase 1 boot path (not just QEMU-direct).

(1) and (2) compound and should go together. (3) gates M5 (the ten forms). (4) gates the fleet-deploy story.

---

_For the full checklist see [`tracking.md`](./tracking.md). For the long-vision see [`vision.md`](./vision.md). For every design decision see [`adrs/`](./adrs/)._
