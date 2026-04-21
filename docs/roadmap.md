# brasa — roadmap

All phase month counts assume 1-2 FTE. Multiply by 2-3 for solo-evenings.
The phases are sequential; later ones are scoped but not scheduled precisely.

## Phase 0 — Design (months 0-3)

**Goal:** the shape of the system is agreed before code is written.

### Deliverables

- [x] Repository exists, public, MIT.
- [x] README, CLAUDE.md, architecture.md, roadmap.md, naming.md, ADRs 0001-0008.
- [x] Workspace Cargo.toml with 8 empty crate skeletons.
- [x] flake.nix providing dev shell.
- [ ] First proof-of-concept: `tronco` boots under QEMU aarch64, prints a typed message via a single syscall, exits cleanly.
- [ ] Proptest suite against the capability ABI in `raiz` — at least the confinement invariant passes.
- [ ] First tatara-lisp form: `(defcapability NetCap …)` parses, type-checks, produces a typed Rust declaration.

### Exit criteria

- ADR-0001 reviewed by ≥1 external reader (open PR, capture comments).
- Capability ABI proptest property green over 10,000 cases.
- QEMU PoC runs reproducibly via `nix run .#brasa-qemu-phase-0`.

## Phase 1 — Minimum kernel (months 3-9)

**Goal:** a typed userspace process exists.

### Deliverables

- `tronco`: scheduler, memory management, IPC primitives on aarch64.
- `raiz`: full cap table, grant/revoke/derive/pass, ~9 cap types.
- `casca`: ~25 typed syscalls (see ADR-0003 for the list).
- `semente`: boots under kasou via `VZEFIBootLoader`, produces `H₀`, hands off.
- `raizame`: records the chain through two levels (semente → tronco).
- `folha::rt`: minimal userspace runtime — a Rust program can link against it, receive a cap bag, issue a syscall, exit.

### Exit criteria

- Two typed userspace processes can IPC via a `EndpointCap<P>` on QEMU.
- `cap_attest_chain()` returns `Vec<[H₀, H₁]>` for a running process.
- Proptest: confinement, monotonic rights, and chain-totality all green.

### Kasou contribution required

`kasou` currently uses `VZLinuxBootLoader`. We need to add `VZEFIBootLoader` support. This is a separate PR against `pleme-io/kasou`. Estimated effort: 2 weeks.

## Phase 2 — Userspace foundation (months 9-15)

**Goal:** the system boots into something usable as a development target.

### Deliverables

- `floresta` (separate repo): init system, reads a manifest produced from `(defsystem …)`, spawns services.
- First drivers (separate repos): `galho-virtio-console`, `galho-virtio-net`, `galho-virtio-blk`.
- `seiva` protocol suite: typed protocols for console, net-device, block-device.
- `sui` integration: boot manifest is a `.nix` file evaluated by sui at build time (not runtime).
- `jabuti-store`: serves `/nix/store/` paths as `StorePathCap` mmap targets.
- Interactive shell: a minimal shell (not bash, not zsh — a new typed one) linked against `folha::rt`, running under `floresta` as a service.

### Exit criteria

- `floresta` launches a service graph declared in `(defsystem phase-2-demo …)`.
- TCP echo server service runs under brasa; cid machine can `nc` into it via the virtio-net bridge.
- Boot chain: semente → tronco → floresta → service, all four measured.

## Phase 3 — First real hardware (months 15-24)

**Goal:** brasa boots on a non-VM machine.

### Deliverables

- Bare-metal Apple Silicon bring-up (or fall back to a Raspberry Pi 5 if Apple Silicon proves blocked).
- USB HID + USB mass storage drivers.
- Ethernet (bcmgenet for Pi5; Apple Silicon ethernet is via USB-C dongle initially).
- Serial console on bare metal.

### Exit criteria

- A physical machine in the fleet runs brasa.
- The machine is registered in `tend` and reachable via VPN.
- `kensa` reports a successful compliance check with the chain rooted at the bootloader hash.

## Phase 4 — Attestation + convergence (months 24-30)

**Goal:** compliance-by-construction is real.

### Deliverables

- `raizame` integrated with `tameshi` signing keys.
- `kensa` has a "brasa native" backend: reads `cap_attest_chain()` directly instead of parsing logs.
- `floresta` runs a FluxCD-style reconciliation loop: reads `(defsystem …)` continuously from a mounted store path, converges the running service graph.
- ProcessTable is native — not a CRD, but a kernel-provided directory of running processes visible via syscall.

### Exit criteria

- A brasa node passes a NIST 800-53 Moderate baseline check with zero external agents.
- The convergence loop auto-remediates a killed service within 5 seconds.

## Phase 5 — Graphics + daily-drive (months 30-42)

**Goal:** one person in the org daily-drives brasa on bare metal.

### Deliverables

- One GPU driver. Two candidate paths:
  - (A) Apple Silicon GPU, leveraging Asahi Linux research. 3-6 person-months by itself.
  - (B) virtio-gpu (VM-only), with one Mesa path. Much cheaper but not bare-metal.
- `garasu` ported to brasa (wgpu backend).
- `madori` + `egaku` compile and render a window on brasa.
- First daily-drive app: `mado` (terminal) or `tobirato` (launcher).

### Exit criteria

- One developer uses brasa as their primary OS for one week.
- `mado` renders at ≥60fps.

## Phase 6 — Fleet + self-host (months 42+)

**Goal:** brasa is a real fleet citizen.

### Deliverables

- x86_64 architecture support.
- Second GPU target.
- Self-host: `nix build .#brasa-image` runs on brasa itself.
- Port of selected `nexus` services (`hanabi`, `kenshi`, `shinka`) to brasa runtime.
- Gradual fleet migration starting with stateless services.

### Exit criteria

- ≥10% of the pleme-io fleet runs brasa.
- A brasa build of brasa is bit-identical to a darwin build of brasa (build determinism).

---

## Open questions before Phase 1

The following must be resolved before Phase 1 ships. Each is tracked as an ADR that does not yet exist.

### Q1: IOMMU — hard or soft?

**Decision needed by:** end of Phase 0.

Options:
- **Hard:** require IOMMU-capable hardware for DMA-safe drivers. Simplifies proofs; excludes some hardware.
- **Soft:** trust drivers on non-IOMMU platforms. More compatibility; weaker guarantees.

**Recommendation:** Hard. Apple Silicon has IOMMU; virtio under kasou has virtual IOMMU; Pi5 has SMMU. We're not losing targets we care about.

### Q2: sui in-kernel or userspace service?

**Decision needed by:** start of Phase 2.

Options:
- **In-kernel:** fast boot, larger TCB, `sui` added to trusted code.
- **Userspace:** slower first boot, smaller TCB, `sui` is just a service.

**Recommendation:** Userspace. The TCB argument dominates; cold-boot Nix evaluation cost is paid once per image build, not per boot — so in-kernel doesn't save what it seems to.

### Q3: Per-core or global run queue?

**Decision needed by:** start of Phase 1 (affects `tronco::sched`).

Options:
- **Per-core with work-stealing:** standard modern design; more code.
- **Global queue with locking:** simpler; doesn't scale past ~4 cores.

**Recommendation:** Per-core with work-stealing from day one. Adding it later is a rewrite.

### Q4: First GPU target?

**Decision needed by:** start of Phase 5.

- Apple Silicon (Asahi-informed) — authentic, gigantic effort.
- virtio-gpu only — cheap, VM-only, delays bare-metal daily-drive.
- PCIe desktop GPU (AMD/NVIDIA/Intel) — Mesa port, full-featured, but takes us off the Apple Silicon first-target story.

**No recommendation yet.** Revisit end of Phase 4.

### Q5: Verification ambition — proptest or Kani/Creusot?

**Decision needed by:** end of Phase 0.

Options:
- **Proptest only:** pragmatic, fast CI, catches most bugs.
- **Kani/Creusot on kernel core:** real formal proofs on scheduler, MM, cap table. Slower CI, authentic verification.

**Recommendation:** Proptest through Phase 2. Pilot Kani on `raiz::rights` algebra in Phase 3. Full formal verification of `tronco::sched` + `raiz` core is a Phase 4 target.

### Q6: First real machine if Apple Silicon bare-metal blocks?

**Decision needed by:** start of Phase 3.

- Raspberry Pi 5 — cheap, documented, slow.
- Apple Silicon MacBook — your actual machine, much harder.
- x86_64 mini PC — unloved arch, but fleet-relevant.

**Recommendation:** Hold this until Phase 3. By then the VM path has matured enough to judge the bare-metal cost honestly.

### Q7: Existing repo strategy — fork or feature-gate?

**Decision needed by:** start of Phase 1 (affects dependency graph).

`tatara-lisp`, `sui`, `kaname` — do we hard-fork them for `no_std` kernel-adjacency, or add `no_std` feature gates to the upstream crates?

**Recommendation:** Feature-gate upstream. Forking fragments the ecosystem; gating lets us benefit from all future upstream work. If gating proves too contorted in practice, fork with a clear "brasa-specific" suffix on each forked crate.

### Q8: When does the first brasa node go into the fleet?

**Decision needed by:** end of Phase 3.

- Option A: as a "pet" node end of Phase 3 (unreliable, fun).
- Option B: as a "cattle" node Phase 4, when attestation is real.

**Recommendation:** Option A. Put one at end of Phase 3, accept it'll get rebuilt weekly. Real fleet adoption waits for Phase 4.

### Q9: Any Linux shim ever?

**Decision:** No. Codified in [ADR-0007](./adrs/0007-no-linux-abi.md). If we ever need to run Linux binaries, we will run them in a Linux VM, not via a shim on brasa. This is deliberate — the moment we have a shim, everyone uses it, and the whole "typed syscalls all the way down" story collapses.

### Q10: Syscall budget — 30, 40, or 50?

**Decision:** 40, per the CLAUDE.md and ADR-0003. seL4 has 11; we allow headroom for Nix-native + attestation + cap ops but hold the line at 40. Crossing it opens a separate ADR.
