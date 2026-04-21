# ADR-0004: tatara-lisp as authoring surface

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

Every OS needs a declarative authoring surface — something between "raw C for drivers" and "a YAML file the kernel parses at runtime." Linux uses Kconfig + devicetree + systemd units + dozens of ad-hoc config files. NixOS uses Nix modules (already a step up). brasa has an opportunity to use a single typed authoring surface for drivers, services, capability types, and whole-system manifests.

The pleme-io prime directive applies here in full force: *every repeatable pattern is a macro*. A new driver should not require 500 lines of boilerplate Rust. A new service should not require a new YAML format. A new capability type should not require changing three files in three crates.

## Decision

All brasa authoring happens in **tatara-lisp**, via `(def…)` forms. The `forja` compiler (existing, in the `tatara` repo) expands these forms into typed Rust IR, which `rustc` and `crate2nix` compile into a kernel image + userspace services.

The kernel **does not parse Lisp at runtime**. All Lisp is compile-time. The kernel loads a pre-compiled typed manifest — one `BootManifest` struct serialized in CBOR — and uses it to spawn the declared initial service graph.

### Four core forms

Each driven by `#[derive(TataraDomain)]` on a Rust struct in the appropriate crate.

#### `(defcapability ...)`

Declares a new capability type. Compiles to a `Cap<NewType>` + associated `Rights` enum + proptest generators + `CapType` impl.

```lisp
(defcapability NetCap
  :ops [:bind :connect :send :recv]
  :rights [:read :write]
  :refine (lambda (cap port)
            (and (< port 65536)
                 (not (reserved-port? port)))))
```

Expands to (conceptually):

```rust
#[derive(TataraDomain, Debug)]
#[tatara(keyword = "defcapability", target = "brasa")]
pub struct NetCap;

impl CapType for NetCap {
    const OPS: &'static [Op] = &[Op::Bind, Op::Connect, Op::Send, Op::Recv];
    const RIGHTS: Rights = Rights::Read.union(Rights::Write);
    fn refine(cap: &Cap<Self>, port: u16) -> bool {
        port < 65536 && !reserved_port(port)
    }
}
```

#### `(defdriver ...)`

Declares a driver. Compiles to a userspace crate (a `galho-*` repo) with a main entry point that receives caps and runs a driver loop.

```lisp
(defdriver :name e1000
           :bus :pci
           :match {:vendor 0x8086 :device 0x100e}
           :caps-requested [(mmio :device-bound)
                            (dma :size 4MB)
                            (irq :any)]
           :protocol net-device
           :impl (rust-crate "galho-e1000"))
```

The `:impl` field points to a Rust crate that provides the concrete driver logic; the `(defdriver …)` form generates the boilerplate around it — PCI matching, cap request protocol, `seiva` endpoint wiring, lifecycle.

#### `(defservice ...)`

Declares a service (daemon-style userspace process).

```lisp
(defservice :name hanabi-bff
            :from (store-path "/nix/store/abc…-hanabi-0.3.1")
            :entry "/bin/hanabi"
            :caps-granted [(net-cap :bind [:tcp 8080] [:tcp 8081])
                           (store-read "/nix/store/def…-hanabi-config")
                           (cpu-budget :cores 2 :class :interactive)]
            :depends-on [postgres-proxy flux-agent]
            :restart :on-failure)
```

Compiles to a `ServiceSpec` struct in the boot manifest, consumed by `floresta` at init.

#### `(defsystem ...)`

The whole-system target state.

```lisp
(defsystem plo
  :arch :aarch64
  :drivers [e1000 nvme apple-gpu]
  :services [hanabi-bff shinka-migrator flux-agent kensa-agent]
  :attest {:baseline :fedramp-moderate
           :signer  "tameshi-release-key"}
  :converge :continuous)
```

This is the entry point. `nix build .#brasa-image` evaluates a `.nix` file that produces a `(defsystem …)` form, which `forja` compiles into a `BootManifest`, which `semente` embeds into the kernel image.

### Compilation pipeline

```
user.lisp                       (authoring surface)
    │
    ▼ tatara::parse
ParsedForm                      (tatara-lisp AST)
    │
    ▼ forja::expand
TypedIR                         (Rust types from #[derive(TataraDomain)] registrations)
    │
    ▼ forja::codegen
Rust source + Nix expressions   (generated artifacts)
    │
    ▼ rustc + crate2nix
Kernel ELF + per-service ELFs   (binary artifacts)
    │
    ▼ semente::pack
brasa.img                        (bootable image)
```

The `(def…)` forms are registered via `#[derive(TataraDomain)]` on Rust structs in:

- `raiz` — registers `(defcapability …)`
- `galho` — registers `(defdriver …)`
- `folha` — registers `(defservice …)`
- `brasa` (top-level) — registers `(defsystem …)`

This matches the existing tatara pattern (`tatara/docs/rust-lisp.md`) where Rust structs export their authoring keyword automatically.

### What the kernel sees

The kernel receives a single typed manifest at boot:

```rust
pub struct BootManifest {
    pub system_name: CStr16,
    pub arch: Arch,
    pub services: &'static [ServiceSpec],
    pub drivers: &'static [DriverSpec],
    pub attest: AttestPolicy,
    pub converge_mode: ConvergeMode,
}
```

No strings to parse. No dynamic structure. A `&'static BootManifest` baked into the kernel image by `semente`.

## Alternatives considered

### YAML / TOML everywhere

Rejected. YAML is Turing-complete by accident and type-unsafe by design. Every YAML config format is a dialect. We have typed configs via shikumi for individual services, but system-wide authoring needs stronger types.

### Nix for the whole thing

Considered. Nix is already our package and derivation language. Some authoring — `.nix` files that produce `(defsystem …)` forms — does live in Nix. But Nix is not well-suited as a general authoring surface for driver behavior; it lacks the macro story and the type-IR story that tatara-lisp provides. Nix *wraps* tatara-lisp in our pipeline, not replaces it.

### Rust itself (no Lisp)

Considered. "Just write it in Rust" eliminates one layer. Rejected because the authoring surface needs macros that compose across domains (capability types refer to driver behavior refers to service specs) — Rust proc macros are not the right tool for this kind of cross-cutting declarative composition. tatara-lisp's `defform` macro-of-macros handles this cleanly.

### A new DSL (non-Lisp)

Considered and rejected. We already have tatara-lisp. It already has `forja`. It already has `#[derive(TataraDomain)]` integration. Inventing a new DSL would be pure duplication.

## Consequences

### Good

- A new capability type, driver, or service is a one-form declaration. Adding `(defcapability NetCap …)` is ~8 lines. Adding it in plain Rust across three crates would be ~200.
- The authoring surface is uniform with the rest of the pleme-io fleet. A developer who has authored a Pangea architecture or a tatara domain already knows how to author for brasa.
- The manifest the kernel loads is *all* typed. Invalid configurations are rejected at compile time, not at first boot.
- Generated code is auditable: `forja --emit-rust < system.lisp > generated.rs`. Anyone can inspect the expansion.

### Bad

- Learning curve: people not familiar with Lisp find the parens strange. The `forja` error messages must be first-class to compensate.
- Debugging generated code: when a generated driver misbehaves, you end up reading expanded Rust. This is not different from macro-heavy Rust projects today, but it's still a cost.
- Bootstrap chicken-egg: we need `forja` and `tatara-lisp` to be stable `no_std`-compatible before we can compile kernel artifacts. This affects the roadmap (Phase 0 must pull in tatara-lisp with `no_std` features).

### Neutral

- All brasa artifacts are reproducible from the `(defsystem …)` source. Lost a build? Re-run `nix build .#brasa-image`. Identical output.

## Verification

1. CI step: `forja --check system.lisp` runs on every PR.
2. Expansion regression test: `forja --emit-rust system.lisp | sha256sum` is pinned in CI; changes to forja that alter expansion for no semantic reason are caught.
3. Round-trip test: `parse(pretty_print(form)) == form` for every form in the corpus.
4. Type-coverage: every `#[derive(TataraDomain)]` must have at least one example in `docs/examples/*.lisp`.

## Open questions

- **`raizame` naming:** noted in [naming.md](../naming.md). If native speakers find it jarring, an amendment renames to `enxerto` (graft) across all artifacts.
- **Macro hygiene across cap types:** if a `(defcapability A)` refers to another capability `B`, does the `forja` expansion handle the reference correctly? Needs a test case by end of Phase 0.

## Amendments

None yet.
