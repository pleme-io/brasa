# ADR-0007: No Linux ABI — ever

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

The single most attractive shortcut in a new-kernel project is: "let's add a Linux syscall shim so we can run existing software." Redox has `relibc`, Fuchsia has `Starnix`, WSL has its own. The argument is compelling — you instantly get access to decades of compiled software, the entire nixpkgs binary cache, every CLI tool anyone has ever written.

We explicitly reject this shortcut. This ADR codifies the rejection.

## Decision

**brasa will never implement a Linux syscall shim.** Not now, not later, not as an optional feature gated behind a flag.

If a workload requires running Linux binaries, it will run them in a Linux VM, not on brasa with a compatibility layer.

## Why

### The shim is a magnet

The moment a Linux syscall layer exists, developer time flows toward it. Every new port takes "the easy path." Every edge case in Linux semantics becomes a bug filed against brasa. Within a year, the typed syscall surface (ADR-0003) is a secondary citizen; the primary interface is a faithful emulation of Linux, with all of Linux's problems.

This has happened to every OS that tried it:

- Redox's `relibc` dominates Redox's mindshare; their native Redox ABI is underused.
- WSL1 was a Linux emulator; it turned into WSL2 — a literal Linux VM — because emulation could never keep up.
- macOS's Mach layer is theoretically capability-based but practically invisible because BSD is the de-facto API.

### Linux semantics contradict brasa invariants

Linux has:

- Ambient authority via UIDs. brasa rejects this (ADR-0001).
- Paths as first-class identifiers. brasa rejects this (ADR-0005).
- Integer `errno` returns. brasa rejects this (ADR-0003).
- Signals as asynchronous process interruption. brasa has no signals.
- `fork` as cheap process creation. brasa attests every spawn (ADR-0002).

A faithful Linux shim would require reintroducing every one of these. A non-faithful shim would frustrate users who expect Linux semantics. Either way, we lose.

### The niche is still there without Linux

brasa's value proposition is not "a place to run old code." It is "a place to run typed, attested, declared-convergent systems." The competitor is not Linux; it is the runtime semantics of ambient-authority Unix. A user who wants Linux semantics already has Linux.

### The costs of the shim are enormous even if we accept the losses

A faithful Linux shim is measured in person-decades. Linux's ABI has hundreds of syscalls, thousands of edge cases, and ~20 years of backwards-compatibility accretion. Even fractional coverage (the 80% of syscalls most programs use) is ~100k lines of shim code. We have no intention of doing this work.

## Alternatives considered

### Optional shim behind a feature flag

Rejected. The ecosystem pressure does not care about feature flags. The moment the shim exists, downstream projects depend on it, and the flag becomes "on by default."

### Shim restricted to specific programs (e.g., "just to run bash")

Rejected. No natural line exists. Any program calls `open()`, `read()`, `write()`, `stat()`, `fork()`, `execve()` — the core Linux API. Supporting one program means supporting all of them.

### Shim as a separate optional service

Considered as a hedge. A userspace service named "penumbra" that provides a Linux-like interface by translating to brasa native syscalls and offering typed caps as file descriptors. This is less objectionable than an in-kernel shim because it doesn't touch the TCB.

Still rejected at this layer. If such a service is ever built, it would be a separate project — not in the `brasa` org, not on the brasa roadmap, and any program running under it would carry an attestation chain that indicates "ran under a compatibility service" (a tameshi signal that compliance tooling can filter on). We anticipate this existing eventually as a side project but commit to not building it ourselves.

### Linux VMs ("run Linux software by running Linux")

This is the correct answer. If you need to run a Linux program, spin up a Linux VM — on brasa's host (if brasa is a VM guest) or eventually on brasa itself (if brasa is bare metal) via a nested-virt path. The VM has its own kernel with its own ambient authority; brasa is not contaminated.

## Consequences

### Good

- The typed syscall surface stays the primary interface. Developer attention concentrates.
- The TCB stays small. No 100k-line shim.
- brasa's invariants (capability confinement, attestation totality, no ambient authority) hold. A shim would force us to violate them.
- We can say clearly what brasa is and is not. "Brasa does not run Linux binaries" is a feature of the product.

### Bad

- Every program must be ported. There is no shortcut for users who want to use existing tools. We accept this.
- Pressure from contributors will exist: "Please add a shim." We will refuse. This ADR is the anchor for that refusal.
- Some workloads that should run on brasa for architectural reasons will choose Linux-on-brasa VM instead of brasa-native. That's fine.

### Neutral

- Porting effort for a given program is domain-specific. A program with minimal file I/O and no fork/exec (most Rust programs) ports trivially. A program that uses the full Unix process model ports badly or not at all. The set of things that ports easily is the set of things we already care about in pleme-io.

## Verification

1. No crate in the brasa workspace depends on anything claiming Linux ABI compatibility.
2. Code review: any PR introducing syscalls that map to Linux semantics 1:1 is challenged in review.
3. Communication: the README and every public talk about brasa states this position explicitly.
4. This ADR is linked from every contributor-facing document. If someone shows up asking "can we add a shim?" the answer is "read ADR-0007, and then we talk."

## What to tell new contributors

> "We don't support Linux binaries. If you need to run Linux software, run it in a Linux VM. The tradeoff is deliberate: we're building something that isn't Linux and can't become Linux. If you want to port a program to brasa, the `folha::rt` runtime + the `casca` syscall surface is the full API. That's it. No escape hatch."

## Amendments

None. This decision is intended to be permanent. Any future attempt to revisit it requires a superseding ADR that addresses every point in the "Why" section above, not merely claims that circumstances have changed.
