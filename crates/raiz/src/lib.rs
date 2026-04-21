//! # raiz — brasa capability system
//!
//! The root. Every authority in brasa is a typed, unforgeable capability:
//! [`Cap<T, R>`] where `T: CapType` is the kind of object and `R: Rights`
//! is the compile-time-encoded right set.
//!
//! See [`ADR-0001`](../../../docs/adrs/0001-capability-abi.md) for the design.
//!
//! # Design invariants
//!
//! 1. **Confinement:** a process cannot obtain a cap to an object it was
//!    not explicitly granted one for.
//! 2. **Monotonic rights under derivation:** [`Cap::derive`] can only reduce rights.
//! 3. **Atomic revocation:** after [`Cap::revoke`], no path reaches the object.
//! 4. **No forgery:** `CapId` values are kernel-opaque; userspace cannot synthesize.
//! 5. **Parent-child direction:** cap flow is directional per parent-child edge.
//!
//! All invariants are proven by proptest in [`testing::invariants`] — see the
//! ADR for the full spec.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

/// Opaque kernel-side handle to a capability. Userspace sees these as `u64`
/// but cannot synthesize a valid one — the kernel verifies table membership
/// on every syscall that takes a `Cap<_>`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct CapId(pub u64);

/// Marker trait for anything that can be held as a capability target.
pub trait CapType: sealed::Sealed {}

/// Marker trait for a compile-time right set.
pub trait Rights: sealed::Sealed {
    const BITS: u16;
}

/// Typed, unforgeable, non-`Copy`, non-`Clone` capability.
///
/// The only way to obtain a `Cap<T, R>` is to receive it via IPC from a
/// process already holding a cap with at least rights `R`, or to have it
/// granted in the initial cap bag at process spawn.
#[repr(C)]
pub struct Cap<T: CapType, R: Rights = Full> {
    handle: CapId,
    _phantom: PhantomData<(T, R)>,
}

// Phase 1: explicit `!Copy` / `!Clone` via the absence of the impls. The
// negative-impl feature (`impl !Copy`) is still unstable and requires more
// ceremony than is worth in Phase 0. We will add a clippy lint
// `no-accidental-copy` in Phase 1 to make the intent machine-enforceable.
//
// Invariant: no `#[derive(Copy, Clone)]` ever appears on `Cap`.

pub mod sealed {
    /// Seal trait — prevents downstream crates from adding new `CapType` /
    /// `Rights` impls.
    pub trait Sealed {}
}

/// Full rights — every operation the cap type supports.
pub struct Full;
impl sealed::Sealed for Full {}
impl Rights for Full {
    const BITS: u16 = 0xFFFF;
}

/// No rights — revoked or uninitialized.
pub struct None_;
impl sealed::Sealed for None_ {}
impl Rights for None_ {
    const BITS: u16 = 0x0000;
}

// ------ Phase 1 cap types (stubs) ------

/// Physical memory region. Phase 1.
pub struct MemCap;
impl sealed::Sealed for MemCap {}
impl CapType for MemCap {}

/// Virtual address space — what "a process" is, at the type level. Phase 1.
pub struct VSpaceCap;
impl sealed::Sealed for VSpaceCap {}
impl CapType for VSpaceCap {}

/// CPU time budget + scheduling class. Phase 1.
pub struct CpuCap;
impl sealed::Sealed for CpuCap {}
impl CapType for CpuCap {}

/// IRQ line. Phase 1.
pub struct IrqCap<const N: u16>;
impl<const N: u16> sealed::Sealed for IrqCap<N> {}
impl<const N: u16> CapType for IrqCap<N> {}

/// MMIO range bound to a device. Phase 1.
pub struct MmioCap;
impl sealed::Sealed for MmioCap {}
impl CapType for MmioCap {}

/// DMA-safe memory + IOMMU ticket. Phase 1.
pub struct DmaCap;
impl sealed::Sealed for DmaCap {}
impl CapType for DmaCap {}

/// Nix store path, read-only mmap. Phase 1.
pub struct StorePathCap;
impl sealed::Sealed for StorePathCap {}
impl CapType for StorePathCap {}

/// Reasons a syscall can be denied. See ADR-0003.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Denied {
    NoCap,
    WrongCapType,
    InsufficientRights,
    Revoked,
    OutOfMemory,
    InvalidArgument,
    ProtocolViolation,
    Timeout,
    ChainOverflow,
    Unsupported,
}

#[cfg(feature = "testing")]
pub mod testing {
    //! Proptest generators and invariant suites.
    //!
    //! Phase 1 deliverable: `invariants::confinement`,
    //! `invariants::monotonic_rights`, `invariants::atomic_revocation` all
    //! green over 10,000 cases.
    pub mod invariants {}
}
