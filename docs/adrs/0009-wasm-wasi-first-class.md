# ADR-0009: WASM/WASI as a first-class runtime

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

[ADR-0007](./0007-no-linux-abi.md) commits brasa to permanently refusing a Linux ABI shim. That decision leaves a real gap: the overwhelming majority of useful software in 2026 is written in languages and idioms that do not trivially target a capability-native Rust-and-Lisp kernel. If brasa has no answer for "how do I run code I wrote in Python / Go / JavaScript / …", the rust/lisp thesis stays confined to pleme-io-internal workloads forever, and the supremacy claim in [`vision.md`](../vision.md) is untestable against the real ecosystem.

The answer is **not** compromise — we will not ship Linux syscalls under any guise. The answer is a second, smaller, cap-compatible runtime that preserves every brasa invariant and accepts code from any language with a WebAssembly backend. That runtime is **WebAssembly + WASI Preview 2 (the component model)**.

## Decision

**brasa treats WASM/WASI as a first-class runtime, peer to the native-Rust runtime.** Every brasa platform service that exposes an interface to userspace (filesystem via `jabuti-store`, network via `galho-virtio-net`'s upper layer, clocks, sockets, streams) publishes both a native `seiva::Protocol<P>` **and** a WASI interface binding that targets the same protocol.

Every WASM component runs inside `wasmtime-on-brasa`, a userspace service that:

1. Holds the caps the component's parent declared in the `(defcomponent …)` manifest.
2. Instantiates the component with typed import bindings.
3. Routes component host-calls through `seiva::Endpoint`s to platform services.
4. Never grants the component any authority beyond the held cap bag.
5. Participates in the `raizame` attestation chain — the component's BLAKE3 hash is a chain link.

At the authoring surface, WASM components are declared in tatara-lisp exactly like native services:

```lisp
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

`forja` expands this into a typed Rust IR consumed by `floresta` at spawn time, and into a compile-time check that the declared `:imports` are covered by the `:caps-granted` set.

## Why WASM is structurally compatible with brasa

The compatibility isn't an accident. WASI Preview 2's component model and brasa's capability ABI are built on the same primitives:

| brasa primitive | WASM/WASI analogue |
|---|---|
| `Cap<T, R>` — unforgeable, non-`Copy`, typed | WASI resource handle — unforgeable, non-`Copy`, typed |
| `casca::Casca` — typed syscall surface | WIT world — typed interface surface |
| `seiva::Protocol<P>` — typed IPC | WASI interface (`wasi:io/streams`, etc.) |
| `raiz::Rights` — compile-time rights | WASI capabilities granted at instantiation |
| BLAKE3 attestation chain | Component hash + supply-chain signature |
| No ambient authority (ADR-0001) | No ambient authority (WASI 0.2 principle) |

The translation layer (`wasmtime-on-brasa`) does essentially one job: **bind imported WASI interfaces to `seiva::Endpoint`s that the parent granted via cap, and refuse any import not covered by a cap.** That's 200-500 lines of glue per WASI interface binding, and the pattern is identical across all of them.

## The WASM runtime — wasmtime-on-brasa

We adopt `wasmtime` (Bytecode Alliance, pure Rust) as the runtime. Reasons:

- **Pure Rust.** No C dependencies, composes naturally with `folha::rt` on brasa.
- **Component model.** Wasmtime is the reference implementation of WASI Preview 2.
- **Sandbox semantics.** Wasmtime's resource-handle model aligns with our cap model without needing adaptation.
- **Maintained + mainstream.** We don't want to own a WASM runtime; we want to integrate one.

Integration shape:

- A new repo: `pleme-io/wasmtime-on-brasa` (separate, under our maintenance, tracks upstream).
- Depends on upstream `wasmtime` as a Cargo dependency.
- Adds brasa-specific glue: a `seiva::Protocol` ↔ WASI interface binding layer per supported WASI interface.
- Packaged as a `galho`-style service: spawned by `floresta`, receives caps + WASM module path, instantiates and runs.
- One `wasmtime-on-brasa` instance per WASM component (isolation boundary = service boundary).

### Initial WASI interface coverage (Milestone M-WASM)

- `wasi:io/streams@0.2.0` — bound to `seiva::Protocol<ByteStream>`.
- `wasi:filesystem/preopens@0.2.0` — bound to `StorePathCap` + `MutableBlobCap`.
- `wasi:sockets/tcp-*@0.2.0` — bound to `NetCap` via the upper-layer network service.
- `wasi:clocks/monotonic-clock@0.2.0` — bound to `casca::time_now`.

Follow-on phases expand coverage (wasi:http, wasi:keyvalue, wasi:random, etc.).

### Not supported initially

- `wasi:cli/environment` — ambient env vars are an ADR-0005 violation; components receive config via typed imports only.
- `wasi:cli/exit` — a component can only exit via returning from its entry; cannot call a kernel exit from inside.
- Ambient stdin/stdout/stderr — every I/O stream is a cap-gated `ByteStream`, not a default handle.

## Alternatives considered

### Roll our own WASM runtime

Rejected. We have no comparative advantage in WASM VM engineering. Wasmtime is mature, fast, and Rust-native. Reinventing is the prime-directive anti-pattern: *"before writing code, check if a macro/library already exists."*

### Pick a different runtime (wasmi, Wasmer, Wasmtime, etc.)

Considered. `wasmi` is smaller but less feature-complete; `Wasmer` has good component support but is less Rust-native and has a history of turbulent governance. `Wasmtime` wins on: pure Rust, component model reference, maintained by Bytecode Alliance, sandbox alignment with our cap model. We could revisit if wasmtime's direction diverges from our needs.

### Run Linux-compatible runtimes inside brasa (Node.js, Python VM, …)

Rejected. These assume POSIX; they'd need a Linux-ish shim to run. Compile-to-WASM is the bridge. Python-on-WASM is a solved problem (CPython has a WASM target; Pyodide; RustPython-WASM); TypeScript-on-WASM via AssemblyScript / JS-on-WASM via ShiftJS / Node.js WASI. The route is always through WASM.

### WASI Preview 1 (the older single-module interface)

Considered and partly supported — wasmtime can execute Preview 1 modules too. But we aim the authoring surface at Preview 2 (components). Preview 1 support exists as a compatibility shim for existing WASM artifacts; new brasa-targeted authoring is Preview 2.

## Consequences

### Good

- **The ecosystem gate opens without compromising the thesis.** Any WASM-targetable language lands on brasa with capability confinement and attestation for free.
- **pleme-io fleet compatibility becomes incremental.** Existing services compile to WASM as an interim step while their native-Rust ports happen.
- **Cross-arch portability for userspace.** A WASM component runs identically on brasa-on-kasou, brasa-on-pi5, brasa-on-x86_64. No per-arch builds for userspace.
- **Third-party code sandboxing is a natural consequence, not a bolted-on feature.** The cap bag at spawn is the entire authority surface.
- **Supply-chain attestation gets stronger.** The attestation chain includes the WASM component hash, which is content-addressed and independent of host toolchain. Reproducing a build is meaningful.

### Bad

- **Runtime overhead.** Wasmtime is fast (~1.3-2x native on CPU-intense workloads) but not free. Native Rust services stay native.
- **Another build pipeline.** Every WASM component has its own language toolchain. We don't own those, and upstream churn is real.
- **Component model is young.** WASI Preview 2 is ~2 years old at writing; interfaces still evolve. We pin specific versions in manifests and track upstream.
- **WIT ↔ `seiva::Protocol` mapping is hand-written per interface.** 200-500 lines of glue per interface. We do build a tatara-lisp macro (`(defwasi-binding …)`) to generate the boilerplate, but edge cases remain case-by-case.

### Neutral

- **Two parallel runtimes (native + WASM) to maintain.** This is deliberate. The native runtime is for things that must be fast or small; WASM for things that must be portable or sandboxed.

## Verification

1. Milestone M-WASM (see `vision.md` and `tracking.md`) is the integration test: a `(defcomponent …)` declared in Lisp, built into a `.wasm` artifact, spawned by `floresta`, reaching `wasi:io/streams` to emit output observable via `galho-virtio-console`.
2. CI gate: every declared `(defcomponent …)` in the brasa corpus must have its `:imports` list verifiable against `wasmtime-on-brasa`'s supported-WASI-interface registry.
3. Attestation invariant: a WASM component's chain link is identical whether the component was built on Darwin, Linux, or brasa itself — content-addressed, build-host-independent.
4. Cap-confinement proptest (extends ADR-0001's suite): a WASM component in wasmtime-on-brasa's sandbox cannot reach any cap not present at its spawn.

## Open questions

- **WASI Preview 3 timing.** Preview 3 adds async semantics and refines the component model further. We adopt when stable; tracking upstream.
- **WASIX** (a superset of WASI aimed at POSIX-adjacency). Rejected for now — it reintroduces ambient-authority patterns we reject. If upstream WASIX drops the ambient pieces, reconsider.
- **GPU access from WASM components.** `wasi:gpu` is in early discussion upstream. We will support when WASI-standardized; a brasa-specific interim path is not worth the divergence.
- **Debugging experience.** WASM debugging is young. Expect time spent on `wasmtime-on-brasa`'s panic / trap observability.

## Amendments

None yet.
