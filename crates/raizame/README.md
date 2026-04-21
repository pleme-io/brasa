# raizame (root-chain) — attestation chain

The woven net of roots. BLAKE3 chains every running process back to `semente`'s measurement of `tronco.elf`.

See [`../../docs/adrs/0002-attestation-chain.md`](../../docs/adrs/0002-attestation-chain.md) for the full design.

## Phase 0 shape

- `BlakeHash`, `ChainInput`, `CapBagDigest`, `ProcessChain` types.
- `ProcessChain::extend` — pure-functional chain extension with `MAX_DEPTH = 16` ceiling.
- `tameshi-compat` feature reserved for the 1:1 mapping into `tameshi::AttestationLayer`.

## The chain formula

```
Hₙ = BLAKE3(Hₙ₋₁ ∥ image_hash ∥ caps_digest)
```

Each link is 32 bytes. Chains cap at depth 16; deeper spawn-trees fail with `Denied::ChainOverflow`.

## Naming note

`raizame` is a compound of *raiz* (root, Portuguese) and *ame* (woven / net / chain — Japanese borrowing, attested in Brazilian-Portuguese through Japanese-Brazilian vocabulary). If native speakers find the portmanteau jarring, the rename candidate is **enxerto** (graft). Tracked in [`../../docs/adrs/0004-tatara-lisp-authoring.md`](../../docs/adrs/0004-tatara-lisp-authoring.md#open-questions).
