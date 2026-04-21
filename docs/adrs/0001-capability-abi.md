# ADR-0001: Capability ABI

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

Every operating system has to answer one question before any other: *how does a process name the things it's allowed to touch?* Unix answered this with the filesystem namespace + UIDs + a small set of descriptor types (fd, pid, signal). That answer was good enough in 1970 and has cost the industry billions of dollars in security incidents since — every ambient-authority bug (SUID, path traversal, `/dev/mem`, `setuid`, `chroot` escape) traces to the original decision to let a process name objects globally.

brasa's core thesis is that this is where Linux compatibility is wrong at the root. No process has any authority except what it holds a typed capability for. The type system — not a runtime check — enforces that invalid authorities are unrepresentable.

## Decision

The brasa kernel uses typed, unforgeable capabilities as the sole mechanism by which processes name resources. All syscalls operate on capabilities. There is no ambient authority, no global filesystem namespace, and no UID/GID model.

### The `Cap<T, R>` type

Defined in the `raiz` crate:

```rust
pub struct Cap<T: CapType, R: Rights = Full> {
    handle: CapId,             // kernel-side table index; opaque to userspace
    _phantom: PhantomData<(T, R)>,
}

impl<T: CapType, R: Rights> !Copy for Cap<T, R> {}
impl<T: CapType, R: Rights> !Clone for Cap<T, R> {}
```

- `CapId` is a 64-bit opaque handle. Userspace cannot synthesize one; the only way to obtain a `Cap<T>` is to receive it via IPC from a process that already holds one with at least the requested rights.
- `!Copy` and `!Clone` at the type level prevent accidental duplication. Duplication happens through the explicit `cap::duplicate(&cap, reduce_rights)` syscall, which always narrows rights — duplication that preserves full rights requires an additional `RightsGrant` cap which is rare.
- Rights are compile-time-encoded via the `Rights` trait; common rights types include `Read`, `Write`, `ReadWrite`, `Execute`, `Revoke`, `Grant`, and composites thereof.

### Cap families (Phase 1 scope)

| Type | Represents | Typical rights |
|------|------------|----------------|
| `MemCap` | A physical memory region | Read, Write, ReadWrite |
| `VSpaceCap` | A virtual address space (a "process") | Grant, Revoke |
| `CpuCap` | CPU time budget + scheduling class | Read (query), Grant (sub-allocate) |
| `IrqCap<N>` | A specific IRQ line | Read (wait), Write (ack) |
| `MmioCap` | An MMIO range bound to a device | Read, Write |
| `DmaCap` | DMA-safe memory + IOMMU ticket | Read, Write |
| `EndpointCap<P: Protocol>` | IPC endpoint for typed protocol `P` | Send, Recv |
| `StorePathCap` | A Nix store path, read-only mmap | Read |
| `ServiceCap<S: Service>` | A running service handle | Read (query), Revoke |

### Cap operations

Five primitive operations, exposed in the `casca` syscall surface:

| Operation | Syscall | Semantics |
|-----------|---------|-----------|
| `grant` | `cap_grant` | Parent grants a held cap to a child process |
| `revoke` | `cap_revoke` | Parent atomically invalidates a cap previously granted |
| `derive` | `cap_derive` | Produce a new cap with *reduced* rights from an existing one |
| `pass` | `cap_pass` | Transfer a cap via IPC message; sender loses the cap, receiver gains it |
| `attest` | `cap_attest_chain` | Return the attestation chain for the caller |

`cap_duplicate` is deliberately not primitive — it composes as `derive(cap, same_rights)`, gated by whether the caller holds a `RightsGrant` meta-cap.

### Core invariants (proven by proptest)

1. **Confinement:** ∀ process P, ∀ object O. If P does not hold a cap to O at time t, no sequence of syscalls from P produces a cap to O at time t' ≥ t except via receipt from another process already holding it.
2. **Monotonic rights under derivation:** `cap_derive(c, r')` requires `r' ⊆ rights(c)`. Any syscall that would violate this fails with `Denied::RightsElevation`.
3. **Atomic revocation:** After `cap_revoke(c)` returns, every process that held a cap derived from `c` observes `Denied::Revoked` on the next use.
4. **No forgery:** `CapId` values produced by userspace (e.g., `unsafe { transmute }`) are rejected at the syscall boundary. The kernel verifies the cap-table entry pointed to by a handle actually belongs to the caller.
5. **Parent-child boundary:** A child cannot grant a cap to its parent. Cap flow is directional per parent-child edge; peers communicate only via endpoints explicitly granted by a common ancestor.

Proptest suites live in `raiz/src/testing/invariants.rs` and run on every PR touching `raiz` or `tronco::syscall`.

## Alternatives considered

### Unix file descriptors

Rejected. FDs are the prototypical ambient-authority mechanism — every process inherits its parent's FD table by default, SUID creates silent authority elevation, and the filesystem namespace is globally shared. brasa's thesis is that this is the wrong foundation.

### Linux-style credentials (UID/GID + capabilities)

Rejected. Linux capabilities (in the `CAP_*` sense) are a patch on top of the UID/GID model — a narrowing filter, not a replacement. They do not help with the confused-deputy problem and they require `getuid()`-style ambient queries.

### Zircon/Fuchsia handles

Considered and partly adopted. Zircon handles are capability-like: unforgeable, non-Copy, transferable. brasa's `Cap<T>` is inspired by Zircon but goes further: rights are compile-time, types are deeply-typed (a `Cap<MemCap, Read>` is a different type than `Cap<MemCap, Write>`), and there is no equivalent of `zx_handle_duplicate` that preserves rights.

### seL4 CNodes

Considered and partly adopted. seL4's guarantees (formal verification, capability confinement) are the inspiration. We deviate in three ways:
1. seL4 uses a general untyped-to-typed retype mechanism; brasa uses typed caps from creation — simpler and less flexible, which we accept.
2. seL4 uses CSpace + VSpace as separate structures; brasa merges them into a single per-process `VSpaceCap` with an integrated cap table.
3. seL4's formal verification is full-kernel; brasa starts with proptest and adds formal tooling (Kani, Creusot) incrementally (per ADR-0009, when written).

### WebAssembly component model capabilities

Considered. WASM's component-model capabilities are interface-level, not resource-level; they solve a different problem (module composition). We take inspiration for the typed-interface story at the IPC layer (`EndpointCap<P>`) but not for the kernel ABI.

## Consequences

### Good

- Confused-deputy attacks become structurally impossible. A process that doesn't hold a cap cannot be tricked into misusing one.
- Security review reduces to auditing cap grants at `(defsystem …)` declaration time, not runtime behavior.
- `kensa` compliance checks become type-level: "this service holds only NIST-approved capability combinations" is a compile-time assertion.
- No SUID, no `setuid`, no privilege escalation primitives. The word "root" does not exist at the kernel level.
- Proof-friendliness: proptest and Kani can actually cover the cap algebra; the same property on a Unix kernel is intractable.

### Bad

- Every existing program must be rewritten. There is no "just port it" — cap-centric APIs are different in kind from FD-centric APIs.
- The `folha::rt` runtime replaces libc; any program that uses raw libc calls (or transitively depends on one via `std`) does not work.
- Debugging and observability have to be redesigned — no `/proc`, no `ps`, no `strace`. We build these from typed primitives in Phase 4.
- IPC becomes mandatory for operations Unix does "inline" (e.g., opening a file is an IPC to the file-server galho that owns the store path). We expect this cost to be offset by aggressive typed-message batching in `seiva`.

### Neutral

- The cap ABI is the single most stable thing in brasa. Breaking changes require ADR supersession, not amendment.

## Verification

1. Proptest suite in `raiz/src/testing/invariants.rs`, ≥ 10,000 cases per invariant, run in CI.
2. Every PR touching `raiz::Cap<…>` or `tronco::syscall` requires an updated proptest property — reviewers block merge without it.
3. Static check in CI: `grep -r "!Copy\|!Clone" crates/raiz/src/` confirms `Cap` remains non-Copy/non-Clone.
4. Kernel-side audit: `tronco::syscall` handlers are the only code paths that construct `Cap<…>` values directly. A clippy lint blocks `Cap::<_, _>::new` outside `raiz::kernel`.
5. Phase 3+ ambition: Kani on `raiz::rights` algebra (deferred to a future ADR).

## Amendments

None yet. When amended, link forward here.
