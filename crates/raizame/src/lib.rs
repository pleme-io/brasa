//! # raizame — brasa attestation chain
//!
//! The woven net of roots. Every process in brasa carries a BLAKE3 chain
//! rooted at [`semente`]'s measurement of `tronco.elf`. The kernel
//! produces the chain; userspace reads it via [`casca::Casca::cap_attest_chain`].
//!
//! See [`ADR-0002`](../../../docs/adrs/0002-attestation-chain.md).

#![cfg_attr(not(feature = "std"), no_std)]

/// 32-byte BLAKE3 digest.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BlakeHash(pub [u8; 32]);

/// Inputs to a single chain extension.
///
/// `Hₙ = BLAKE3(parent ∥ image ∥ caps)` — see ADR-0002.
#[derive(Clone, Copy, Debug)]
pub struct ChainInput {
    pub parent: BlakeHash,
    pub image: BlakeHash,
    pub caps: CapBagDigest,
}

/// BLAKE3 digest of the canonical CBOR encoding of a cap bag.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapBagDigest(pub [u8; 32]);

/// The per-process chain state, held in the kernel, readable via syscall.
///
/// `MAX_DEPTH = 16`; spawns at depth 17 fail with `Denied::ChainOverflow`.
#[derive(Clone, Copy, Debug)]
pub struct ProcessChain {
    pub links: [BlakeHash; Self::MAX_DEPTH],
    pub depth: u8,
}

impl ProcessChain {
    pub const MAX_DEPTH: usize = 16;

    /// Extend the chain with a new link. Returns `None` if extending would
    /// exceed `MAX_DEPTH`.
    #[must_use]
    pub fn extend(&self, input: ChainInput) -> Option<Self> {
        if usize::from(self.depth) >= Self::MAX_DEPTH {
            return None;
        }
        let mut hasher = blake3::Hasher::new();
        hasher.update(&input.parent.0);
        hasher.update(&input.image.0);
        hasher.update(&input.caps.0);
        let hash = BlakeHash(*hasher.finalize().as_bytes());

        let mut next = *self;
        next.links[usize::from(self.depth)] = hash;
        next.depth += 1;
        Some(next)
    }
}

#[cfg(feature = "tameshi-compat")]
pub mod tameshi_compat {
    //! 1:1 mapping from `raizame::ChainInput` → `tameshi::AttestationLayer`.
    //! Phase 1 integration.
}
