# ADR-0003: Syscall surface

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

Linux has roughly 400 syscalls. Most are historical, many are redundant, several are known-problematic (ambient authority, confused-deputy, integer errno). Every syscall is a line of kernel code that can have a bug — Linux CVEs trace disproportionately to the syscall layer.

seL4 has 11 syscalls. Fuchsia's Zircon has about 180. Darwin (XNU) has ~500.

brasa's syscall surface is the single most important determinant of kernel TCB size, provability, and velocity. A small surface is easier to verify, audit, port, and evolve. A large surface is harder on all four axes. We cap it explicitly.

## Decision

**Hard cap: 40 syscalls.** Phase 1 target: ~25. Crossing 40 requires a separate ADR debating the cap itself, not merely adding a syscall.

Every syscall:
- Is defined in the `casca` crate as a trait method on `trait Casca`.
- Takes typed arguments (caps, `CapId`, small-POD structs) — never raw pointers from userspace, never C-style null-terminated strings.
- Returns `Result<T, Denied>` where `T` is either a new typed cap, a small POD, or `()`.
- Has a proptest generator for its arguments in `casca::testing`.
- Has a kernel-side implementation in `tronco::syscall::<name>`.

### The Phase 1 syscall list (25)

Grouped by concern. Numbers are the syscall-id integer used on the `svc` trap path.

#### Capability core (6)

| # | Name | Purpose |
|---|------|---------|
| 01 | `cap_grant` | Grant a held cap to a child process |
| 02 | `cap_revoke` | Atomically invalidate a previously granted cap |
| 03 | `cap_derive` | Produce a new cap with reduced rights |
| 04 | `cap_pass` | Transfer a cap via IPC (consumed from sender) |
| 05 | `cap_attest_chain` | Return the caller's attestation chain |
| 06 | `cap_inspect` | Query cap type + rights (no dereference) |

#### Memory (4)

| # | Name | Purpose |
|---|------|---------|
| 10 | `mem_alloc` | Allocate `N` pages; returns `Cap<MemCap, ReadWrite>` |
| 11 | `mem_map` | Map a `MemCap` into the caller's VSpace at a given address |
| 12 | `mem_unmap` | Unmap a `MemCap` from the caller's VSpace |
| 13 | `mem_share` | Split a `MemCap` in two (returns two caps covering disjoint ranges) |

#### Process (4)

| # | Name | Purpose |
|---|------|---------|
| 20 | `proc_spawn` | Spawn a new process with a given image, cap bag, VSpace |
| 21 | `proc_exit` | Exit the caller; non-returning |
| 22 | `proc_yield` | Voluntarily yield the remainder of the CPU slice |
| 23 | `proc_wait` | Wait on a `ServiceCap<S>` until it exits |

#### IPC (5)

| # | Name | Purpose |
|---|------|---------|
| 30 | `ipc_send` | Send a typed message on an `EndpointCap<P>` (non-blocking) |
| 31 | `ipc_recv` | Receive a typed message on an `EndpointCap<P>` (blocking with timeout) |
| 32 | `ipc_call` | Send-then-receive on the same endpoint (blocking) |
| 33 | `ipc_endpoint_new` | Create a new endpoint, returns a `(sender, receiver)` cap pair |
| 34 | `ipc_poll` | Poll multiple endpoints (limited multiplexing primitive) |

#### Device (3)

| # | Name | Purpose |
|---|------|---------|
| 40 | `dev_mmio_map` | Map an `MmioCap` into caller's VSpace as device memory |
| 41 | `dev_irq_wait` | Block on an `IrqCap<N>` until the IRQ fires |
| 42 | `dev_irq_ack` | Acknowledge an IRQ (re-arms the line) |

#### Store (2)

| # | Name | Purpose |
|---|------|---------|
| 50 | `store_open` | Open a `StorePathCap` as a read-only `MemCap` |
| 51 | `store_stat` | Query metadata of a `StorePathCap` |

#### Time (1)

| # | Name | Purpose |
|---|------|---------|
| 60 | `time_now` | Monotonic nanoseconds since boot |

**Total:** 25 syscalls in Phase 1. Headroom to 40 is for:
- A few Phase 4 attestation convenience primitives.
- Phase 5 GPU/DMA shared-buffer ops (`dma_alloc`, `dma_map`).
- Phase 6 cross-node IPC for fleet-mode (anchored to kakuremino transport).

### Shape: every syscall is typed

No syscall takes a `char*`. No syscall takes a path. No syscall has variadic-integer arguments with meaning determined by some other integer. Concrete shape in `casca`:

```rust
pub trait Casca {
    fn cap_grant(&self, target: Cap<VSpaceCap>, what: Cap<dyn CapType>) -> Result<(), Denied>;
    fn cap_revoke(&self, what: Cap<dyn CapType>) -> Result<(), Denied>;
    fn cap_derive<T, R, R2>(&self, cap: &Cap<T, R>, rights: R2) -> Result<Cap<T, R2>, Denied>
    where
        T: CapType, R: Rights, R2: Rights + ReducedFrom<R>;
    // ...
    fn mem_alloc(&self, pages: u32, rights: Rights) -> Result<Cap<MemCap, Rights>, Denied>;
    fn proc_spawn(&self, image: Cap<StorePathCap>, caps: CapBag, vspace: Cap<VSpaceCap>) -> Result<Cap<ServiceCap<Unknown>>, Denied>;
    fn ipc_send<P: Protocol>(&self, ep: &Cap<EndpointCap<P>, Send>, msg: P::Message) -> Result<(), Denied>;
    // ...
}
```

The trait is implemented once in `tronco::syscall::Kernel` and once in `casca::userspace::Svc` (which marshals to the `svc` instruction). Both implementations share the argument types — there is no `ioctl(2)` style dynamic dispatch.

### The `Denied` type

```rust
pub enum Denied {
    NoCap,               // caller doesn't hold the required cap
    WrongCapType,        // caller holds a cap but wrong type
    InsufficientRights,  // caller holds the right type but wrong rights
    Revoked,             // cap was revoked
    OutOfMemory,         // kernel could not allocate
    InvalidArgument,     // out-of-range or malformed typed argument
    ProtocolViolation,   // IPC message doesn't match endpoint protocol
    Timeout,             // blocking operation exceeded timeout
    ChainOverflow,       // process tree depth exceeded MAX_DEPTH
    Unsupported,         // syscall reached but current kernel build omits it
}
```

Not an integer `errno`. Not a bitfield. A closed enum, pattern-matched at call sites.

## Alternatives considered

### Larger syscall surface (Linux/Darwin style)

Rejected. The cost of every additional syscall is: one more kernel code path, one more fuzzing target, one more proof obligation, one more thing that can break ABI compatibility. If our 25 syscalls don't express everything, the answer is "compose them" or "add a userspace service with its own typed protocol," not "add a syscall."

### seL4-minimal (~11)

Considered. seL4 gets to 11 by pushing almost everything into the cap model — even operations that feel separate (thread control, VSpace management) are cap invocations on specific cap types. We considered this but chose 25 because it's more approachable for developers joining the project, and the security properties we care about (confinement, attestation) are already enforced by the cap ABI regardless of syscall count.

### ioctl-style dynamic dispatch

Explicitly rejected. `ioctl` is the prototypical anti-pattern: a single syscall that demultiplexes on an integer into arbitrary functionality, with argument shapes not visible to the type system. The entire point of brasa's ABI is that this does not happen.

### Per-cap-type syscalls (grow the surface with each new cap type)

Considered. If we added a syscall per cap type operation, we'd hit 40+ by Phase 2. We reject this because the cap model already generalizes: a new cap type reuses `cap_grant`, `cap_revoke`, `cap_derive`, etc. Cap-type-specific behavior lives in the protocol attached to that cap (e.g., the `EndpointCap<NetDevice>` protocol handles ARP/DHCP, not the kernel).

## Consequences

### Good

- TCB is small. 25 syscalls × ~50 lines average = ~1200 lines of syscall-handler code in `tronco::syscall`. All reviewable.
- Fuzzer targets are enumerable. A stateful fuzzer that drives typed `Casca` methods covers 100% of the syscall surface.
- ABI is stable: adding a syscall requires an ADR. Removing or changing one requires a superseding ADR. The surface doesn't drift.
- Porting brasa to a new arch is bounded: implement 25 trap handlers.

### Bad

- Programs that expect to make tens of syscalls per high-level operation (e.g., an HTTP server: open, read, write, close) must restructure to use one IPC call per operation (talk to the network service via `EndpointCap<NetDevice>`).
- We must build composable primitives carefully. If a pattern requires 6 syscalls where one would do, developers will push for a new syscall — which we refuse. This creates pressure on us to make sure the primitives are right.

### Neutral

- 25-40 is small enough to memorize. This is a feature, not a limitation.

## Verification

1. Compile-time enumerated: `casca::SyscallId` is a closed enum; `#[repr(u16)]` and the kernel dispatch table is a `[Option<fn>; 256]` that's only populated at known indices.
2. CI check: count lines matching `^    fn ` in `casca::Casca`; fail if > 40.
3. PRs adding a syscall must include a diff to this ADR. A GitHub check enforces this.
4. Proptest generators exist for every syscall's arguments; the count of generators must match the count of trait methods.

## Amendments

None yet. Expected amendments in later phases:
- Phase 4: attestation convenience primitives (+2-3 syscalls, ETA 30/40).
- Phase 5: DMA shared-buffer ops (+2 syscalls, ETA 32/40).
- Phase 6: cross-node IPC (+2-3 syscalls, ETA ~35/40).

If we hit 38 and still have appetite, a new ADR raises the cap. Without that ADR, additions stop at 40.
