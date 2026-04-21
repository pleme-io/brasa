# brasa — tracking

> The checklist from here to supremacy. Every box maps to a verifiable outcome.
> When you tick a box, update [`status.md`](./status.md) in the same commit.

This document is organized by **milestone** (M-prefixed, coarse-grained, each maps to a meaningful externally-visible change) and by **phase** (numbered, matches [`roadmap.md`](./roadmap.md), a time-bounded work unit). Milestones can span phases; phases can contain multiple milestones.

Legend: `[x]` = proven green today, `[ ]` = open, `[~]` = partially complete, `[?]` = design still uncertain.

## Top-line milestones — the spine

| M | Name | Phase | Status | One-line |
|---|------|-------|--------|----------|
| M0 | Boot | 0 | `[x]` | `brasa-bin` prints typed UART banner under QEMU, halts cleanly. Proven `2026-04-21`, commit `120bf16`. |
| M1 | First syscall | 1 | `[ ]` | A userspace process on brasa makes `cap_attest_chain()` via `svc`, receives a typed `ChainView`. |
| M2 | First Lisp-authored driver | 1-2 | `[ ]` | `galho-virtio-console` authored as `(defdriver …)`, `forja`-compiled, `floresta`-spawned. |
| M3 | Confinement proptest-green | 1 | `[ ]` | `raiz::testing::invariants::confinement` green over 10,000 cases. |
| M-WASM | First WASM component | 3-4 | `[ ]` | A `.wasm` component runs under `wasmtime-on-brasa`, reaches WASI output, chain-attested. |
| M4 | First fleet node | 3 | `[ ]` | A brasa node joins the pleme-io fleet (tend + VPN + Flux), kensa-compliant. |
| M5 | Ten forms land | 2-5 | `[ ]` | All core tatara-lisp forms have codegen + Rust + proptest + Nix. |
| M6 | Self-host | 6 | `[ ]` | `nix build .#brasa-image` succeeds on a brasa node; BLAKE3-identical to Darwin build. |
| M7 | External contributor | 6+ | `[ ]` | Non-pleme-io author lands a `(defdriver …)` using only the ten forms; zero `unsafe` added. |
| M8 | Formal-verification floor | 4-5 | `[ ]` | `raiz::rights` algebra proven via Kani at full state-space scale. |
| M∞ | Supremacy | ∞ | `[ ]` | Non-negligible portion of the pleme-io fleet on brasa; no competitor on safety/mutability. |

---

## Phase 0 — Design (months 0-3) — **GREEN (founding session)**

### 0.1 — Repository skeleton
- [x] `pleme-io/brasa` public with MIT license + CLAUDE.md + README.
- [x] Workspace `Cargo.toml` with 8 kernel-tight crates (`tronco`, `raiz`, `seiva`, `casca`, `galho`, `folha`, `semente`, `raizame`) + Phase-0 PoC `brasa-bin`.
- [x] `flake.nix` dev shell resolves on Apple Silicon macOS + Linux via fenix stable + qemu + llvm.
- [x] `.cargo/config.toml` target-scoped rustflags for `aarch64-unknown-none`.

### 0.2 — ADR spine
- [x] ADR-0001 Capability ABI. Accepted.
- [x] ADR-0002 Attestation chain. Accepted.
- [x] ADR-0003 Syscall surface (hard cap 40, Phase-1 list of 25). Accepted.
- [x] ADR-0004 tatara-lisp as authoring surface. Accepted.
- [x] ADR-0005 Nix store as filesystem. Accepted.
- [x] ADR-0006 First target: Apple Silicon + kasou. Accepted.
- [x] ADR-0007 No Linux ABI — ever. Accepted.
- [x] ADR-0008 Open source from day zero. Accepted.
- [x] ADR-0009 WASM/WASI as a first-class runtime. Accepted.

### 0.3 — Sibling repos scaffolded
- [x] `pleme-io/floresta` — userspace init types + design docs, pushed.
- [x] `pleme-io/galho-virtio-console` — scaffolded, pushed.
- [x] `pleme-io/galho-virtio-net` — scaffolded, pushed.
- [x] `pleme-io/galho-virtio-blk` — scaffolded, pushed.

### 0.4 — Upstream bring-up (kasou)
- [x] `BootConfig` enum with `Linux { … } | Efi { … }` variants.
- [x] `VZEFIBootLoader` + `VZEFIVariableStore` plumbed through `config.rs`, `builder.rs`.
- [x] All existing kasou tests pass (46 / 46). New tests added for EFI paths.
- [x] `kikai` call-site updated to `BootConfig::Linux { … }`, CI-green.

### 0.5 — Phase-0 PoC
- [x] `crates/brasa-bin/` bin crate with `#![no_main]`, aarch64 `global_asm!` entry.
- [x] Linker script targeting QEMU virt load address (`0x4008_0000`).
- [x] PL011 UART writer at `0x0900_0000`.
- [x] Panic handler emits typed location.
- [x] `cargo build -p brasa-bin --target aarch64-unknown-none --release` succeeds.
- [x] `qemu-system-aarch64 -machine virt … -kernel …` produces expected 266-byte banner.

### 0.6 — Exit criteria (all green → Phase 0 complete)
- [x] Kernel boots under QEMU with typed output.
- [x] Workspace compiles with no lint warnings.
- [x] ADRs reviewed by ≥1 reader (author; external review still welcome pre-M1).
- [x] [`status.md`](./status.md) exists and is up to date.
- [x] [`tracking.md`](./tracking.md) (this file) exists.

---

## Phase 1 — Minimum kernel (months 3-9)

### 1.1 — tronco core (aarch64)
- [ ] Physical frame allocator (bump + slab) in `tronco::mm`.
- [ ] 4-level page tables with 4K/2M/1G page support.
- [ ] Virtual address space creation / teardown.
- [ ] Per-core run queue in `tronco::sched` (work-stealing deferred; simple FIFO ok for M1).
- [ ] Exception vector table (EL1 synchronous + IRQ + FIQ + SError).
- [ ] `svc` entry decode path.
- [ ] Timer IRQ handler tied into scheduler preemption.
- [ ] Per-core startup via PSCI CPU_ON.

### 1.2 — raiz capabilities
- [ ] `Cap<T, R>` non-`Copy`/`Clone` enforced by lint or negative impl.
- [ ] Kernel-side cap table with atomic revoke.
- [ ] `cap_grant` / `cap_revoke` / `cap_derive` / `cap_pass` / `cap_inspect` primitives.
- [ ] `Rights` bit-lattice with compile-time `ReducedFrom` trait.
- [ ] Phase-1 cap types (`MemCap`, `VSpaceCap`, `CpuCap`, `IrqCap<N>`, `MmioCap`, `DmaCap`, `EndpointCap<P>`, `StorePathCap`, `ServiceCap<S>`) defined + used.

### 1.3 — casca syscall surface (25 calls)
- [ ] All 25 Phase-1 syscalls declared on `Casca` trait with real typed signatures.
- [ ] Kernel-side implementations in `tronco::syscall` for each.
- [ ] Userspace-side `casca::userspace::Svc` issuer (aarch64 `svc` instruction).
- [ ] CI count-check: `Casca` trait method count ≤ 40.

### 1.4 — seiva IPC
- [ ] `Protocol` trait + three initial protocols: `SystemConsole`, `BootInfo`, `Supervisor`.
- [ ] Endpoint creation (`ipc_endpoint_new`).
- [ ] Typed send / recv / call over `EndpointCap<P>`.
- [ ] Cap-passing through a typed message variant.

### 1.5 — raizame attestation
- [ ] BLAKE3 chain produced at every `proc_spawn`.
- [ ] Per-process `ProcessChain` stored in the VSpaceCap metadata.
- [ ] `cap_attest_chain()` syscall returns the caller's chain.
- [ ] `MAX_DEPTH = 16` enforced.

### 1.6 — M1 — First syscall
- [ ] `brasa-bin` extended with a minimal userspace process link-edited against `folha::rt`.
- [ ] Process issues `cap_attest_chain()`, receives `ChainView`.
- [ ] ChainView rendered to UART via `SystemConsole` protocol.
- [ ] Expected observed output: `chain depth=2: H₀ = abc123…, H₁ = def456…`.

### 1.7 — M3 — Confinement proptest
- [ ] `raiz::testing::model::CapTable` host-side reference model.
- [ ] Proptest generator for operation sequences (`Op::Grant`, `Op::Revoke`, `Op::Derive`, …).
- [ ] `invariants::confinement` property: no sequence produces a cap for P that P wasn't granted.
- [ ] Proves green over ≥10,000 cases in CI.

### 1.8 — kasou brasa guest boot
- [ ] Package `brasa-bin` as UEFI PE (aarch64-unknown-uefi re-target).
- [ ] Integration test in `kasou`: boot a brasa PE via `VZEFIBootLoader` in a VM.
- [ ] Capture serial output, verify banner text matches expected.

### 1.9 — Phase 1 exit criteria
- [ ] Two userspace processes on brasa IPC via `EndpointCap<P>` on QEMU.
- [ ] `cap_attest_chain()` returns 2-link chain for a running process.
- [ ] Confinement / monotonic-rights / chain-totality proptests all green.
- [ ] Kasou guest boot integration test passes in CI.

---

## Phase 2 — Userspace foundation (months 9-15)

### 2.1 — floresta init
- [ ] Binary build path using brasa target triple.
- [ ] Parses `BootManifest` CBOR, verifies self-hash.
- [ ] Topological service-graph spawn.
- [ ] Reconcile-loop tick at configurable interval.
- [ ] Cap revocation on service exit.
- [ ] `ExitNotification` over `Supervisor` endpoint.

### 2.2 — First driver — galho-virtio-console
- [ ] virtio-mmio device enumeration.
- [ ] Feature negotiation (`F_SIZE`, `F_MULTIPORT`).
- [ ] RX / TX vring with pre-allocated DMA buffers.
- [ ] Publishes `Endpoint<VirtioConsole>` to floresta.
- [ ] Replaces the Phase-0 raw PL011 UART writer.

### 2.3 — Second driver — galho-virtio-blk
- [ ] PCI enumeration (virtio-blk modern IDs).
- [ ] Request queue plumbing with 4K sector granularity.
- [ ] `VirtioBlk` protocol via `Endpoint<VirtioBlk>`.
- [ ] Flush semantics correct (client sees durability only after `FlushDone`).

### 2.4 — Third driver — galho-virtio-net
- [ ] Feature negotiation (`F_MAC`, `F_MRG_RXBUF`).
- [ ] RX / TX vring with mergeable RX buffers.
- [ ] L2-frame pass-through over `Endpoint<VirtioNet>`.
- [ ] No IP stack; TCP/UDP lives in a separate service (phase 2.7).

### 2.5 — jabuti-store
- [ ] New repo `pleme-io/jabuti-store`.
- [ ] Backed by `galho-virtio-blk` block device (initial) or ramdisk (boot-0).
- [ ] Serves `/nix/store/` paths as `StorePathCap` mmap targets.
- [ ] Content-hash verification at open time.
- [ ] `MutableBlobCap` issuance against a factory cap.

### 2.6 — M2 — First Lisp-authored driver
- [ ] `brasa-forja-ext` crate / repo registers `(defdriver …)` as a tatara-lisp domain.
- [ ] Existing `galho-virtio-console` Rust body becomes `:impl` of a Lisp `(defdriver virtio-console …)` form.
- [ ] `forja` expansion produces working Rust, compiles, runs identically.
- [ ] Round-trip integration test: commit Lisp, see driver binary built by CI.

### 2.7 — Network service
- [ ] New repo `pleme-io/rede-core` — smoltcp-wrapped TCP/UDP stack.
- [ ] Runs as a `(defservice …)` above `galho-virtio-net`.
- [ ] Exposes typed `NetCap` with bind/connect rights.
- [ ] First end-to-end test: TCP echo on brasa, reachable from host via the kasou bridge.

### 2.8 — Interactive shell
- [ ] Minimal typed shell (not bash, not zsh) running on brasa.
- [ ] Spoken over `SystemConsole` endpoint.
- [ ] Can enumerate held caps (`caps`) and run a short scripted program.

### 2.9 — Phase 2 exit criteria
- [ ] `(defsystem phase-2-demo …)` → compiles → boots → service graph runs.
- [ ] TCP echo from cid workstation lands on brasa guest and replies.
- [ ] Boot chain: semente → tronco → floresta → services, all four measured.

---

## Phase 3 — First real hardware (months 15-24)

### 3.1 — First bare-metal bring-up
- [ ] Decide: Raspberry Pi 5 vs Apple Silicon bare-metal vs x86_64 mini-PC. (ADR-TBD before start.)
- [ ] Port `semente` to the chosen platform's boot protocol.
- [ ] Port `tronco::arch` MMU + timer + IRQ controller.
- [ ] USB HID driver for keyboard input.
- [ ] USB mass storage driver (or NVMe/SD as appropriate).
- [ ] Ethernet driver.

### 3.2 — M4 — First fleet node
- [ ] Brasa node registered in `tend` (pleme-io workspace config).
- [ ] VPN (`mamorigami`) link registered for the node.
- [ ] `kensa` agent service runs; compliance check passes with chain rooted at `semente.elf` hash.
- [ ] Observable from pleme-io fleet dashboards (via shinryu).

### 3.3 — M-WASM (part 1) — wasmtime-on-brasa
- [ ] New repo `pleme-io/wasmtime-on-brasa`.
- [ ] Embeds upstream wasmtime. Compiles for brasa target.
- [ ] Spawns as a `(defservice …)`. Receives a module `StorePathCap` + cap bag.
- [ ] Basic WASI 0.2 imports: `wasi:io/streams`, `wasi:clocks/monotonic-clock`.
- [ ] A trivial "hello" WASM component reaches `streams` output, visible via console.

### 3.4 — Phase 3 exit criteria
- [ ] Brasa boots on a physical machine in the pleme-io fleet.
- [ ] Node passes a kensa compliance check with the bootloader hash as root.
- [ ] At least one WASM component runs under `wasmtime-on-brasa`.

---

## Phase 4 — Attestation + convergence (months 24-30)

### 4.1 — Real tameshi integration
- [ ] `raizame::tameshi_compat` landed with bidirectional mapping.
- [ ] `sekiban` can consume brasa attestation chains without modification.
- [ ] `kensa` runs natively on brasa, reading chains via `cap_attest_chain`.

### 4.2 — FluxCD-style reconciliation
- [ ] `floresta` reconciler reads updates from a mounted `StorePathCap`.
- [ ] Drift detection auto-restarts failed services.
- [ ] Image-hash change triggers graceful restart.
- [ ] Canary/blue-green strategies pluggable via `ReconcileStrategy`.

### 4.3 — ProcessTable as kernel primitive
- [ ] `tronco::sched::ProcessTable` exposes a kernel-visible list of running processes.
- [ ] Readable via `cap_inspect` on a factory cap.
- [ ] Snapshot semantics (atomic, point-in-time).
- [ ] Orphan reaping + zombie detection parity with the pleme-io ProcessTable CRD model.

### 4.4 — M-WASM (part 2) — first-class authoring
- [ ] `(defcomponent …)` tatara-lisp form registered.
- [ ] `forja` checks component imports ⊆ granted caps.
- [ ] CI pipeline: author commits `.wasm` + `(defcomponent)` → `nix build .#image` → brasa reboot → running component.
- [ ] Attestation: component BLAKE3 hash appears in chain, verifiable offline.

### 4.5 — M8 foundation — Kani pilot
- [ ] `raiz::rights` algebra compiled into Kani proof harness.
- [ ] Monotonic-rights-under-derivation proven at full state-space scale.
- [ ] CI gate: Kani proof runs on PRs touching `raiz::rights`.

### 4.6 — Phase 4 exit criteria
- [ ] brasa node passes NIST 800-53 Moderate baseline check with zero external agents.
- [ ] Reconciler auto-remediates a killed service within 5 seconds.
- [ ] WASM component authoring end-to-end lifecycle works for the pleme-io team.
- [ ] Kani-proof cleared for at least one `raiz` component.

---

## Phase 5 — Graphics + daily-drive (months 30-42)

### 5.1 — Graphics path
- [ ] Decide GPU route: Apple Silicon native (Asahi-informed) vs virtio-gpu-only vs PCIe. (ADR-TBD.)
- [ ] `garasu` ported to brasa — abstracts the chosen GPU behind a typed pipeline.
- [ ] `madori` + `egaku` compile against `folha::rt`.
- [ ] One pleme-io app (`mado` terminal or `tobirato` launcher) renders a window on brasa.

### 5.2 — Session + user model
- [ ] `(defsession …)` tatara-lisp form + `kenshou-on-brasa` service.
- [ ] Capability-based session-scoped isolation (no UIDs).
- [ ] Per-session mutable blobs for user data.

### 5.3 — Ten-forms completeness (M5)
- [ ] All ten core forms (`defcapability`, `defdriver`, `defservice`, `defprotocol`, `defsystem`, `defpolicy`, `defblob`, `defstrategy`, `defbaseline`, `defnamespace`) have codegen + Rust + proptest + Nix.
- [ ] Documentation + examples for each form.
- [ ] At least one real brasa service authored purely through forms (no hand-written Rust driver code).

### 5.4 — Phase 5 exit criteria
- [ ] One pleme-io developer uses brasa as their primary OS for one week.
- [ ] `mado` renders at ≥60fps on the chosen GPU target.
- [ ] M5 green.

---

## Phase 6 — Fleet + self-host (months 42+)

### 6.1 — x86_64 arch
- [ ] `tronco::arch::x86_64` port (MMU, IDT, APIC, timer).
- [ ] `semente` x86_64 UEFI variant.
- [ ] CI matrix extended to x86_64-unknown-none + x86_64 EFI boot test.

### 6.2 — Self-host (M6)
- [ ] `nix build .#brasa-image` succeeds on a brasa node.
- [ ] Output is BLAKE3-identical to the Darwin build.
- [ ] Nix daemon runs as a brasa service.

### 6.3 — nexus service ports
- [ ] `hanabi` ships as both a native brasa service and a WASM component.
- [ ] `kenshi` + `shinka` operators ported or re-written (likely with substantial redesign given brasa's different approach).
- [ ] Gradual fleet migration begins with stateless services.

### 6.4 — Phase 6 exit criteria
- [ ] ≥10% of the pleme-io fleet runs brasa.
- [ ] Brasa-on-brasa builds reproducibly.
- [ ] M7 ticked: external contributor landed a driver using only forms.

---

## Phase 7+ — supremacy (∞)

### 7.1 — Full fleet adoption
- [ ] ≥80% of pleme-io fleet on brasa.
- [ ] Linux-on-brasa (via nested virt or kasou equivalent on brasa itself) available for residual workloads that refuse to port.

### 7.2 — M8 — full-formal verification floor
- [ ] `tronco::sched` correctness proven via Kani / Creusot at full scale.
- [ ] `raiz::cap_table` confinement proven at full scale.
- [ ] CI gate: formal-verification tier runs on every `raiz`/`tronco` PR.

### 7.3 — Ecosystem (M7 generalized)
- [ ] Non-trivial outside-pleme-io community exists: third-party galhos on GitHub, community-authored components.
- [ ] brasa documentation published as a book (web + print).
- [ ] Conference presence: one talk, one paper.

### 7.4 — M∞ — supremacy
- [ ] No competitor OS sits where brasa sits on the safety/mutability axis.
- [ ] Compliance audits for pleme-io infrastructure complete by type-check, not interview.
- [ ] Mutability cadence: hours-to-ship a new subsystem, with proofs.
- [ ] Safety cadence: CVE class for every remaining bug category is proven unreachable or explicitly out-of-scope.

---

## How this document stays honest

Rules:

1. **Only tick a box when the claim is reproducible by a command in `status.md`.** A box ticked "green" without a reproducible command is a bug in this file.
2. **Every tick is committed with the artifact it describes.** No ticking-before-merging.
3. **When an ADR changes, the tracking lines that reference it get checked against the amendment.** Superseded rows get struck through, not deleted.
4. **Tracking deltas are part of every session summary.** If a session ticks boxes, the commit message lists them.
5. **Vision and tracking are a pair: when vision changes, tracking changes within the same PR.**

## Open questions (tracked here, not in ADRs yet)

These are issues that need decisions but not yet ADR-worthy:

- **Phase 1 IOMMU policy.** Hard-require IOMMU-capable hardware for DMA-safe drivers, or soft-trust non-IOMMU drivers? Favor hard.
- **sui in-kernel or userspace?** Favor userspace (smaller TCB). ADR pending end of Phase 0 review.
- **Per-core scheduler initial shape.** Work-stealing from day 1 (more code, scales later) or single global queue (simpler, doesn't scale). Favor work-stealing.
- **First GPU target (Phase 5).** Apple Silicon native, virtio-gpu-only, or desktop PCIe? Defer decision to end of Phase 4.
- **Verification ambition.** Proptest-only through Phase 3, Kani pilot in Phase 4, full-formal in Phase 7. Pace may accelerate if early wins land.
- **WASIX.** Reject in the short term (too much ambient-authority). Revisit if upstream drops the POSIX-ish parts.
- **Rename `raizame` → `enxerto`.** Pending native-speaker review of the neologism. Tracked in [ADR-0004 open questions](./adrs/0004-tatara-lisp-authoring.md).

## Amendments

- `2026-04-21` — initial tracking doc created after Phase 0 proven-boot. WASM/WASI first-class added (ADR-0009). WASM/WASI-specific lines added in Phase 3 and Phase 4.
