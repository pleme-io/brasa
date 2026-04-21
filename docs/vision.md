# brasa — the vision

> The rust/lisp line, fully realized.
> Safety by construction. Mutability by design.
> The standardization of computing, pushed to Ring 0.

This document is the north star. Everything in the roadmap, every ADR, every crate in this workspace, and every sibling repo (`floresta`, `galho-*`, `jabuti-store`, …) is accountable to this file. When a design decision disagrees with the vision, we update the vision explicitly (as an amendment below) — we do not silently drift.

## The thesis

Every operating system that has ever shipped has chosen one of three positions on the **safety / mutability** axis:

| Position | Example | Cost |
|---|---|---|
| **Safe, rigid** | seL4, formally-verified research kernels | You can prove everything, but you can't change anything. Pushing a new driver is a publication. |
| **Mutable, unsafe** | Linux, Windows, macOS | You can ship anything overnight, but anything can break anything. 50 years of CVEs. |
| **Neither** | 1970s timesharing kernels, embedded RTOSes | Neither provable nor easy to evolve. |

Nobody has shipped a kernel that is both **provably safe** and **structurally mutable**. The assumption — implicit in every OS engineering culture — is that safety and mutability trade off: the more proofs you have, the less you can change; the more you can change, the fewer proofs survive.

**brasa's thesis is that this assumption is false.** The trade-off exists only when the authoring surface and the verified surface are the same surface. When they are separated — **mutable authoring compiling to safe typed code** — both properties compose.

The mechanism is the **rust/lisp line**:

- **tatara-lisp is the authoring surface.** Declarative, macro-composed, human-mutable. A driver, a service, a capability type, a whole system are all `(def…)` forms. The surface is as fluid as any dynamic language.
- **Rust is the verified surface.** `forja` compiles every Lisp form into typed Rust IR. The kernel loads typed, proven, no-`errno` code. The surface is as rigid as seL4.
- **The compiler is the only path between them.** There is no runtime Lisp interpreter in the kernel. There is no "escape to C" or "unsafe block nobody reviews." The Lisp authors; the Rust executes; the compiler enforces the bridge.

This is the rust/lisp line. brasa's supremacy thesis is that **when that line is fully realized — when every concept in the system has both a Lisp authoring surface and a Rust verified surface, with `forja` the sole path between — the OS is simultaneously more mutable than Linux and more provable than seL4.**

## What "fully realized" means concretely

Every concept in brasa has, at Phase ∞:

1. **A typed Rust implementation** in one of the workspace crates (`raiz`, `tronco`, `seiva`, `casca`, `folha`, `galho`, `semente`, `raizame`) or a sibling repo (`floresta`, `galho-*`, `jabuti-store`).
2. **A `#[derive(TataraDomain)]` registration** exposing it as a Lisp authoring keyword.
3. **A `(def<concept> …)` form** in the tatara-lisp surface.
4. **A set of invariants** expressible as proptest/Kani properties.
5. **A role in the attestation chain** — the boot chain records the use of this concept in every process that invokes it.
6. **A place in the Nix build pipeline** via `forja` expansion.

Authoring a new kernel primitive becomes one PR across three repos, not a decade of committee work.

## Concept inventory — the full rust/lisp line

The supremacy end state has a Lisp keyword + typed Rust surface + proptest properties + Nix build integration for **each of the following concepts**:

| Concept | Lisp keyword | Rust surface | Verifies |
|---|---|---|---|
| Capability type | `(defcapability NetCap …)` | `raiz::Cap<NetCap, Rights>` | Confinement, monotonic rights |
| Rights set | `(defrights Readable …)` | `raiz::Rights` impl | Bit-lattice algebra |
| Syscall | `(defsyscall mem_alloc …)` | method on `casca::Casca` | Argument typing, 40-cap, attestation preservation |
| IPC protocol | `(defprotocol VirtioNet …)` | `seiva::Protocol<VirtioNet>` | Message-shape totality, cap-passing semantics |
| Driver | `(defdriver e1000 …)` | `impl galho::Driver for E1000` | Cap-request validity, lifecycle totality |
| Service | `(defservice hanabi-bff …)` | `folha::ServiceSpec` | Dependency acyclicity, cap-bag validity |
| System state | `(defsystem plo …)` | `floresta::BootManifest` | Global invariants across all services |
| User session | `(defsession kenshou …)` | `kenshou::SessionCap` | Auth-chain totality, cap-isolation per-session |
| Compliance policy | `(defpolicy fedramp-moderate …)` | `kensa::Policy` | Baseline satisfaction across running services |
| Namespace | `(defnamespace /nix/store …)` | `jabuti::NamespaceCap` | Read-only-ness, content-addressing |
| Mutable blob | `(defblob host-identity …)` | `jabuti::MutableBlobCap` | Owner-scoped, BLAKE3-identified |
| Reconciliation strategy | `(defstrategy canary …)` | `impl floresta::ReconcileStrategy` | Convergence termination, action validity |
| Attestation baseline | `(defbaseline tameshi-mod …)` | `raizame::AttestPolicy` | Root signer validity, chain depth |
| Network topology | `(deftopology fleet-mesh …)` | typed in `mamorigami-core` | CIDR non-overlap, key rotation, no-full-tunnel |
| Schedule class | `(defsched interactive …)` | `raiz::SchedClass` + `tronco::sched` | Bounded preemption, RT budget monotonic |
| Graphics pipeline | `(defpipeline brasa-wgpu …)` | `garasu::Pipeline` ported to brasa | GPU cap validity, no DMA escape |
| WASM component | `(defcomponent my-wasi-app …)` | `wasmtime-on-brasa` service | Import graph ⊆ granted caps, memory sandboxed, BLAKE3 attestation |
| WASI interface binding | `(defwasi-binding wasi:filesystem …)` | `seiva::Protocol<…>` impl | Cap-handle mapping totality, no ambient authority leak |

At Phase ∞, **every one of these has all four artifacts** and is composable into every other one via typed Rust channels backed by Lisp composition.

## Safety + mutability compose — the proofs

Mutability without giving up proofs is the whole game. The three mechanisms that make the composition work:

### 1. Compile-time Lisp expansion

`forja` is a compile-time operation. A new `(defdriver …)` form produces Rust code that `rustc` and `crate2nix` compile. The kernel never parses Lisp at runtime. Safety proofs attach to the typed Rust output — the Lisp source is just convenient authoring. You can rewrite every Lisp form overnight without weakening a single proof.

### 2. Typed composition, not name-based

When `(defservice X :depends-on [Y])` names Y, `forja` resolves Y to a typed `ServiceCap<Y>` at compile time. If Y doesn't exist, the kernel image doesn't link. Renames, deletions, signature changes — all caught at compile time, not at first boot. The safety proofs never see a stale reference.

### 3. Attestation-as-structure

Every process carries its chain back to the bootloader. Changing the running system doesn't invalidate the proofs — it produces new attestation links that are themselves proven. You can mutate the running fleet aggressively; each mutation is a new proof, not a broken one.

## Standardization as liberation, not restriction

The pleme-io prime directive — *"everything repeatable becomes a macro or a library; duplication is a bug"* — is typically defensive engineering. In brasa it becomes offensive: **the more the rust/lisp line is standardized, the more a single author can mutate the OS.**

Every macro in tatara-lisp, every typed Rust primitive in `raiz`/`casca`/`seiva`, every `mk<Thing>` Nix builder is a force multiplier. A contributor who understands the ten core forms — `defcapability`, `defdriver`, `defservice`, `defprotocol`, `defsystem`, `defpolicy`, `defblob`, `defstrategy`, `defbaseline`, `defnamespace` — can compose any userspace subsystem on brasa in hours, with proofs attached.

This is the end of the traditional kernel-contributor ecosystem, where you spend years learning the idiosyncrasies of one OS to write one driver. In brasa, the ten forms are the idiosyncrasy, and they are the same forms everywhere in pleme-io: infrastructure, apps, fleet tooling, compliance, and now the kernel.

## What supremacy looks like from a user's seat

A developer on a brasa fleet at Phase ∞:

```lisp
;; Declare a new service. 12 lines. That's the whole delta.
(defservice mojinet-edge
  :from (store-path "/nix/store/abc…-mojinet-0.4.2")
  :entry "/bin/mojinet"
  :caps-granted [(net-cap :bind [:tcp 443])
                 (store-read "/nix/store/def…-mojinet-tls")
                 (cpu-budget :cores 1 :class :interactive)
                 (mutable-blob :name sessions :max-size 512MB)]
  :depends-on [postgres-proxy kensa-agent]
  :restart :on-failure
  :attest {:baseline :fedramp-moderate})
```

They commit this to git. FluxCD sees the new `(defsystem)` manifest. `forja` compiles it. `crate2nix` builds it. `semente` produces a new image, measured into the attestation chain. `kasou` reboots the affected node (seconds). `floresta` converges — spawns mojinet-edge with exactly the declared caps, refuses to give it anything else. `kensa` verifies the attestation chain roots at the pleme-io release signing key. `raizame` records the chain link.

The service runs. It is safe: every syscall it makes is typed, every cap it holds is revocable, every exit path is observed. It is mutable: the developer can rewrite the whole thing tomorrow with a new commit, and the cycle repeats in seconds.

**The developer never wrote a line of unsafe Rust. The developer never touched `make menuconfig`. The developer never opened systemd-analyze. The kernel never ran an unverified byte.**

That is supremacy.

## WASM/WASI as a first-class runtime

The rust/lisp line gives us a native surface of overwhelming expressiveness for the **kernel** and **platform services**. But a kernel alone doesn't run the world. Most of the useful code written in 2026 lives in other languages, in other ecosystems, authored by people who do not know tatara-lisp and will never learn Rust. Ignoring them is a failure of the thesis.

The answer is **not** a Linux ABI — [ADR-0007](./adrs/0007-no-linux-abi.md) is permanent. The answer is **WebAssembly with the WASI component model as a first-class brasa runtime.**

### Why WASM/WASI fits brasa better than Linux fits Linux

- **WASI Preview 2 handles *are* capabilities.** The WASI 0.2 component model is built on typed resource handles: a module can only touch a file, a socket, a clock if it has been granted a handle to one. No ambient authority. No global namespace. The model is structurally compatible with `raiz::Cap<T, R>`.
- **WASM is memory-safe by construction.** Modules run in a linear-memory sandbox; no pointer can escape. This is a complementary proof to brasa's capability confinement.
- **WASM is language-polyglot.** Rust, Go, Python, JavaScript, TypeScript, Swift, C, C++, AssemblyScript, Zig — all target WASM. Any of them, on brasa, gets capability confinement + BLAKE3 attestation + typed IPC **for free**.
- **WASI doesn't assume POSIX.** Unlike emulating Linux, where every syscall is a fight against ambient authority, WASI was designed with capability security in mind from Preview 2 onward. It does not fight our model.
- **Components compose typed interfaces.** WASI interfaces (`wasi:io`, `wasi:filesystem`, `wasi:sockets`, `wasi:http`, `wasi:clocks`, …) are defined in WIT (WebAssembly Interface Types), which is conceptually a typed IPC protocol — identical in spirit to `seiva::Protocol<P>`.

### The brasa↔WASM bridge

The architecture is one `galho`-style service — a WASM runtime service — plus a typed mapping layer:

```
┌──────────────────────────────────────────────────────────────┐
│  WASM component (authored in Rust/Go/Python/TS/…)            │
│  imports: wasi:filesystem, wasi:sockets, app-specific        │
└──────────────────────────────────────────────────────────────┘
                          │ WIT interfaces
                          ▼
┌──────────────────────────────────────────────────────────────┐
│  wasmtime-on-brasa (userspace service)                        │
│  Embeds wasmtime (pure Rust runtime)                          │
│  Translates each WASI import → seiva::Endpoint<Protocol>      │
└──────────────────────────────────────────────────────────────┘
                          │ typed IPC (seiva protocols)
                          ▼
┌──────────────────────────────────────────────────────────────┐
│  Platform galhos (virtio-blk, virtio-net, jabuti-store, …)    │
│  Provide real implementation for WASI interfaces              │
└──────────────────────────────────────────────────────────────┘
                          │ syscalls (casca::Casca)
                          ▼
┌──────────────────────────────────────────────────────────────┐
│  tronco kernel                                                │
└──────────────────────────────────────────────────────────────┘
```

A WASM component's import graph maps 1:1 to `Cap<T>`s granted at launch. If the component imports `wasi:sockets/tcp-bind`, it requires a `NetCap` with bind rights. If it imports `wasi:filesystem/preopens`, it requires `StorePathCap`s for exactly the paths the manifest declares. The component cannot exceed its import graph; the runtime cannot exceed the caps the kernel granted. The caps compose.

### The Lisp authoring surface for WASM

```lisp
;; A WASM component authored by someone who has never touched Rust.
;; They shipped a `.wasm` file; we run it.
(defcomponent mojinet-edge
  :from (store-path "/nix/store/abc…-mojinet-0.4.2.wasm")
  :runtime wasmtime
  :imports [wasi:io/streams@0.2.0
            wasi:filesystem/preopens@0.2.0
            wasi:sockets/tcp-bind@0.2.0
            wasi:clocks/monotonic-clock@0.2.0]
  :caps-granted [(net-cap :bind [:tcp 443])
                 (store-read "/nix/store/def…-mojinet-tls")
                 (cpu-budget :cores 1 :class :interactive)]
  :restart :on-failure
  :attest {:baseline :fedramp-moderate})
```

`forja` checks at compile time that:
1. Every `:imports` entry has a matching seiva protocol on the brasa side.
2. Every `:caps-granted` entry covers the caps implied by the imports (no import without a backing cap).
3. The component file exists at the declared store path and its content-hash matches.

Runtime:
1. `floresta` spawns the `wasmtime-on-brasa` service with the WASM module path + the cap bag.
2. `wasmtime-on-brasa` instantiates the component, binds each import to the corresponding `seiva::Endpoint`.
3. The component's host-calls go through typed IPC to platform galhos — never directly to syscalls, never ambient.
4. The BLAKE3 attestation chain records the component's hash: `… → floresta → wasmtime-on-brasa → <wasm-module-hash>`.
5. `kensa` compliance verifies the chain without knowing anything about WASM — the chain is just bytes.

### What WASM/WASI gives brasa

- **Ecosystem coverage without ABI compromise.** Every language with a WASM backend runs on brasa, confined, attested, capability-bound.
- **A serious answer to "but I can't port my Python script."** Compile it to WASM/WASI; done.
- **Cross-cluster portability.** A WASM component runs on a brasa fleet node, on a kasou VM on a developer laptop, on a Raspberry Pi, on an x86_64 server — same binary, different cap grants.
- **Sandboxed third-party code by default.** Unlike Linux where a package gets your whole uid, a brasa WASM component gets *exactly* the caps its manifest declares.
- **A bridge for the pleme-io fleet too.** Existing pleme-io services (hanabi, kenshi, shinka, nexus, …) can ship to brasa as WASM components initially, while the native-Rust port happens over time. Zero-day-one compatibility with our own fleet.

### What it does not give brasa

- **Not native performance for CPU-intense code.** WASM has runtime overhead (typically 1.3-2x native). Latency-critical services stay native.
- **Not a way to run Linux binaries.** WASM is not Linux. A binary must be compiled to WASM/WASI; it is not a `./a.out`-drop. Linux binaries remain the domain of a Linux VM.
- **Not an excuse to skip the rust/lisp work.** WASM is the compatibility layer for the external ecosystem; the core brasa services (kernel, drivers, init, Nix store, attestation) are Rust + Lisp and will remain so.

### Roadmap integration

WASM/WASI first-class support is **Milestone M-WASM**, lands in Phase 3-4. The lightweight version (embed `wasmtime` as a userspace service, pass caps at spawn time, no component-model-import auto-mapping) is Phase 3. The full thing (`(defcomponent …)` authored in Lisp, `forja` checking imports against cap grants, `wasmtime-on-brasa` as a standard-issue brasa service) is Phase 4. Full WASI Preview 3 / WASIX tracking is Phase 5+.

See [`tracking.md`](./tracking.md) for sub-milestones and [ADR-0009](./adrs/0009-wasm-wasi-first-class.md) for the design commitment.

## What brasa is not chasing

To be explicit about the tradeoffs we reject:

- **Not chasing Linux-compatibility ergonomics.** If you want to run Docker images compiled for Linux, run Linux. brasa will host Linux VMs on brasa eventually, but the brasa native surface does not compromise toward Linux. See [ADR-0007](./adrs/0007-no-linux-abi.md).
- **Not chasing microsecond latency above all else.** Our scheduler is capability-based and will be slower than Linux CFS on adversarial benchmarks. The safety + mutability trade absorbs this. For workloads where latency dominates, stay on Linux or an RTOS.
- **Not chasing "mainstream adoption."** brasa is pleme-io's kernel first. Wider adoption comes naturally if the thesis is right; we don't market for it.
- **Not chasing one-size-fits-all.** brasa on a kasou VM guest, brasa on bare metal Apple Silicon, brasa on an embedded Pi5, brasa on an x86_64 server — each of these is a legitimate target. We do not pretend they have the same constraints.

## Supremacy milestones — the spine

These are the big markers. The full tracker lives in [`tracking.md`](./tracking.md).

### Milestone M0 — boot (achieved 2026-04-20)

Kernel image builds, boots under QEMU aarch64 virt, writes typed UART banner, halts cleanly. **Proof that the toolchain, linker, and boot-drop address compose.** See [`status.md`](./status.md).

### Milestone M1 — first syscall

`tronco::syscall::cap_attest_chain` is the first typed syscall. A userspace process linked against `folha::rt` calls it and receives its two-link chain (`semente → tronco → folha-process`). **Proof that the rust/lisp line reaches userspace.**

### Milestone M2 — first Lisp-authored driver

`galho-virtio-console` authored as `(defdriver virtio-console …)`, compiled by `forja` into typed Rust, spawned by `floresta` from a minimal `(defsystem)` manifest. **Proof that the authoring surface works end-to-end.**

### Milestone M3 — proptest-green capability confinement

`raiz::testing::invariants::confinement` proven green over 10,000 randomized cap-operation sequences. **Proof that the safety thesis is machine-checkable.**

### Milestone M-WASM — first WASM component runs

A `.wasm` binary (authored in any language that targets wasm32-wasip2) is spawned via `(defcomponent …)`, receives caps, makes a host-call that flows through `wasmtime-on-brasa` → `seiva::Endpoint` → platform galho, and produces observable output. Its BLAKE3 hash is part of the attestation chain and passes a `kensa` policy check. **Proof that the ecosystem gate is open without abandoning the thesis.**

### Milestone M4 — first fleet node

A brasa node joins the pleme-io fleet (tend sync, VPN registration, Flux-observed). Passes a `kensa` compliance check rooted at the bootloader hash. **Proof that the mutability thesis composes with real operations.**

### Milestone M5 — the ten forms land

All ten core tatara-lisp forms (`defcapability` through `defnamespace`) have `forja` codegen + Rust surface + proptest suites + Nix build integration. **Proof that the rust/lisp line is fully-realized at the authoring layer.**

### Milestone M6 — self-host

`nix build .#brasa-image` runs on a brasa node. Reproducible. BLAKE3-identical to the Darwin build. **Proof that brasa is a real kernel, not a toy.**

### Milestone M7 — external contributor

A developer outside pleme-io lands a `(defdriver …)` for a new hardware class, using only the ten forms. Zero unsafe Rust written. **Proof that the thesis generalizes.**

### Milestone M8 — formal verification floor

`raiz::rights` algebra proven via Kani (or equivalent) at the full state-space scale. **Proof that the safety thesis is full-formal, not just proptest-green.**

### Milestone M∞ — supremacy

A non-negligible portion of the pleme-io fleet runs brasa. Compliance is structural. Mutability is hours-to-ship. Safety is proved. No competitor on the safety/mutability axis. **Proof by existence.**

## Amendments

This vision is a living document. Amendments are appended here with date + rationale.

_No amendments yet. First amendment will be dated and signed._

---

_For where we are today, see [`status.md`](./status.md). For the tracked checklist of every sub-milestone, see [`tracking.md`](./tracking.md). For the architecture, see [`architecture.md`](./architecture.md). For every design decision, see [`adrs/`](./adrs/)._
