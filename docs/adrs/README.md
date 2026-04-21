# Architecture Decision Records

Every load-bearing design decision in brasa gets an ADR. ADRs are numbered monotonically, never renumbered, and once `Accepted` are never silently edited — further changes come as amendments or superseding ADRs.

## Index

| # | Title | Status |
|---|-------|--------|
| 0001 | [Capability ABI](./0001-capability-abi.md) | Accepted |
| 0002 | [Attestation chain](./0002-attestation-chain.md) | Accepted |
| 0003 | [Syscall surface](./0003-syscall-surface.md) | Accepted |
| 0004 | [tatara-lisp as authoring surface](./0004-tatara-lisp-authoring.md) | Accepted |
| 0005 | [Nix store as filesystem](./0005-nix-store-as-filesystem.md) | Accepted |
| 0006 | [First target — Apple Silicon via kasou](./0006-first-target-apple-silicon-kasou.md) | Accepted |
| 0007 | [No Linux ABI — ever](./0007-no-linux-abi.md) | Accepted |
| 0008 | [Open source from day zero](./0008-open-source-from-day-zero.md) | Accepted |
| 0009 | [WASM/WASI as a first-class runtime](./0009-wasm-wasi-first-class.md) | Accepted |

## Status values

- **Proposed** — under discussion in a PR, not yet merged.
- **Accepted** — merged, governs the project.
- **Amended** — some specifics have changed; see superseding ADR number at the top.
- **Superseded** — replaced entirely by a later ADR.
- **Rejected** — considered and chosen not to do; kept as historical record.

## Format

Each ADR follows this shape:

```
# ADR-NNNN: Title
Status: Accepted | Proposed | …
Deciders: names
Date: YYYY-MM-DD

## Context
What is the problem? What forces are at play?

## Decision
What did we choose?

## Alternatives considered
What did we reject, and why?

## Consequences
What follows from this decision — both good and bad?

## Verification
How will we know this decision is being respected in the code?
```

## How to propose a new ADR

1. Pick the next unused number.
2. Copy an existing ADR as a template.
3. Write under the format above. Be honest about tradeoffs.
4. Open a PR. Get at least one pair of eyes.
5. Merge after discussion settles; status goes from Proposed → Accepted on merge.
