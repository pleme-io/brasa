# ADR-0002: Attestation chain

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

Linux's answer to "is this process trustworthy?" is SELinux or AppArmor — runtime policy engines that read labels off processes and files and make allow/deny decisions. This is bolt-on security: policy is a separate artifact, enforcement is a separate mechanism, and audit is a log you read afterwards.

brasa's thesis is that attestation is not policy — it is identity. A running process *is* its attestation chain. If it can't produce a BLAKE3 chain rooted at the bootloader, it doesn't exist. Not "isn't allowed to run" — literally doesn't exist, because there is no mechanism by which the kernel would have launched it without creating the chain link.

This works for brasa because we already have `tameshi` (core attestation library), `sekiban` (K8s admission webhook), `kensa` (compliance engine), and `inshou` (Nix gate CLI) — all BLAKE3-based, all typed in Rust. brasa extends the same Merkle tree into the boot sequence and the process table.

## Decision

Every process in brasa has a BLAKE3 attestation chain, computed at spawn time, stored in the kernel, readable by the process itself via a typed syscall. The chain is structural — there is no code path in the kernel that produces a process without producing a chain entry.

### The chain formula

```
H₀ = BLAKE3(tronco.elf)                           -- produced by semente
H₁ = BLAKE3(H₀ ∥ floresta.elf ∥ initial_caps)     -- produced by tronco when spawning floresta
…
Hₙ = BLAKE3(Hₙ₋₁ ∥ image.elf ∥ initial_caps)      -- produced when parent spawns child
```

Each `Hᵢ` is a 32-byte BLAKE3 digest. The input format is defined in the `raizame` crate as:

```rust
pub struct ChainInput {
    pub parent: BlakeHash,
    pub image: BlakeHash,           // hash of the loaded image
    pub caps: CapBagDigest,          // hash of the initial cap bag (canonical CBOR encoding)
}
```

`CapBagDigest` is BLAKE3 of a canonical CBOR serialization of the cap bag at spawn time. This includes the cap type, the rights bits, and (for path-bound caps) the target hash — but not the `CapId` (which is a runtime-assigned handle and not attestation-relevant).

### Where chains live

Per-process state in the kernel holds:

```rust
pub struct ProcessChain {
    pub links: [BlakeHash; MAX_DEPTH],
    pub depth: u8,
}
```

`MAX_DEPTH` is 16. Process trees deeper than 16 are not architected for; the kernel refuses to spawn at depth 17. This cap is deliberate: arbitrarily deep chains are a signature of shell-like spawn patterns we explicitly reject.

### How userspace reads it

One syscall, `cap_attest_chain`, returns the caller's chain as a `&[BlakeHash]`. Phase 1 scope; see ADR-0003.

```rust
pub fn cap_attest_chain() -> Result<ChainView, Denied>;

pub struct ChainView {
    pub links: [BlakeHash; MAX_DEPTH],
    pub depth: u8,
}
```

A process can also request the chain of a target identified by a `ServiceCap<S>` — provided it holds the cap. This is how compliance agents verify peer services.

### Signature verification — NOT in the kernel

The kernel does not verify signatures. It only produces chains. Verification is a userspace concern — either offline (in `kensa`-on-alicerce compliance runs) or at runtime in a dedicated attestation service that holds the `tameshi` signing key.

This is deliberate. Signature verification is expensive and involves key management; keeping it out of the kernel keeps the kernel's TCB small. The kernel produces a structural, unforgeable trail; whether any particular chain is *trusted* is a policy decision that lives in userspace.

### Integration with tameshi

`tameshi` is already the pleme-io canonical attestation library. brasa's chain format is a subset of tameshi's `AttestationLayer` format: specifically, a linear chain where each node has exactly one parent and one image. Existing tameshi tooling (`sekiban`, `kensa`, `inshou`) can consume brasa chains unchanged.

Concretely: `raizame::ChainInput → tameshi::AttestationLayer` is a 1:1 mapping documented in `raizame/src/tameshi_compat.rs`.

## Alternatives considered

### IMA/EVM (Linux)

Linux's Integrity Measurement Architecture hashes files at open time and stores measurements in the kernel. Considered and rejected: IMA is bolt-on (optional, often disabled), reads files by path (ambient authority via the filesystem), and requires an external policy engine to make use of the measurements. brasa chains are structural and always present.

### TPM-based attestation

TPM-extending every process's PCR would compose with our chain, but TPM operations are slow (~ms) and TPM availability varies across hardware. We do not require TPM. We do intend to support *anchoring* the boot measurement (`H₀`) in a TPM PCR for hardware-rooted attestation in Phase 4. This is additional, not substitutional.

### Capability hashes (as implemented in some research kernels)

Some kernels hash the cap bag as the process identity. We do this too — the cap bag digest is part of `ChainInput` — but we additionally hash the image and chain it to the parent. A cap-only identity is susceptible to "identical cap bags, different code" confusion; the image hash fixes this.

### Block-based attestation (every cap operation extends the chain)

Considered and rejected: makes the chain unbounded and unverifiable. Our chain is a boot-time structural fact, not a runtime audit log. Audit logs (for observable behavior) are a separate system in `floresta`.

## Consequences

### Good

- Every process has cryptographic provenance by construction; there is no "unsigned" code path.
- `kensa` compliance checks over brasa are structural: "every running process descends from a tameshi-signed release" is a type-level fact.
- No need for AppArmor/SELinux/audit: the chain *is* the audit trail.
- Supply-chain attacks become visible: if a compromised image is loaded, its chain will not verify against tameshi-signed roots, and a kensa agent running in userspace will detect the divergence.

### Bad

- `fork(2)`-style cheap process creation is impossible: every spawn must hash the image. This is a deliberate cost — the Unix fork/exec pattern is one of the things we are leaving behind. Spawn is O(image-size) in BLAKE3 cost, which is tractable but not Linux-fork-fast.
- Chain depth is capped at 16. Shell-style deep trees don't work. Services that need to spawn-and-wait use IPC with a worker pool, not recursive spawning.
- Dynamic code loading (JIT, eval) cannot be chain-verified. We do not support dynamic code loading for services; interpreters run under their own chain link and are trusted through that link.

### Neutral

- Chains are ~32 * 16 = 512 bytes per process. Cheap.
- Chain production is O(BLAKE3(image)) which on Apple Silicon is ~1GB/s — adding ~1ms per 1MB of image. Negligible for service images (typically < 10MB).

## Verification

1. Proptest in `raizame/src/testing/chain_invariants.rs`:
   - Chain totality: every process created has a `depth > 0`.
   - Chain consistency: `chain[depth-1] == BLAKE3(chain[depth-2] ∥ image ∥ caps)`.
   - Chain bound: `depth ≤ MAX_DEPTH`.
2. Integration test: boot brasa in QEMU, spawn 10 processes, verify each chain via tameshi tooling externally.
3. CI gate: PRs that modify `tronco::spawn` must include a test asserting chain extension.
4. Kensa integration test (Phase 4): a compliance check passes end-to-end from bootloader hash to running service.

## Amendments

None yet.
