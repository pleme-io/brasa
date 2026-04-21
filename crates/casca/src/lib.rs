//! # casca — brasa syscall ABI
//!
//! The bark. The single typed surface userspace touches to reach the kernel.
//! The [`Casca`] trait declares the full syscall set; `tronco::syscall`
//! provides the kernel-side implementation, and `casca::userspace` provides
//! the userspace issuer that traps via `svc` (aarch64) or `syscall` (x86_64).
//!
//! See [`ADR-0003`](../../../docs/adrs/0003-syscall-surface.md) for the
//! syscall list and the hard cap of 40.

#![cfg_attr(not(feature = "std"), no_std)]

// Re-exported so downstream users can import `Denied`/`Rights` from casca
// without also needing raiz. Phase 1 expands as Cap/CapType appear on syscall
// signatures in the `Casca` trait below.
#[allow(unused_imports)]
pub use raiz::{Cap, CapType, Denied, Rights};

/// The full syscall surface. Phase 1 target: ~25 methods. Hard cap: 40.
///
/// Implemented once in `tronco::syscall::Kernel` (kernel-side) and once
/// in `casca::userspace::Svc` (userspace issuer).
pub trait Casca {
    // Cap core (Phase 1 — 6 syscalls). Full signatures land with the kernel
    // implementation; Phase 0 just reserves the trait.

    // fn cap_grant(&self, ...) -> Result<(), Denied>;
    // fn cap_revoke(&self, ...) -> Result<(), Denied>;
    // fn cap_derive(&self, ...) -> Result<Cap<_, _>, Denied>;
    // fn cap_pass(&self, ...) -> Result<(), Denied>;
    // fn cap_attest_chain(&self) -> Result<ChainView, Denied>;
    // fn cap_inspect(&self, ...) -> Result<CapInfo, Denied>;

    // Memory (4), Process (4), IPC (5), Device (3), Store (2), Time (1).
    // See ADR-0003 for the full list and numbering.
}

/// Syscall identifier — the integer passed via the trap instruction.
///
/// Phase 1 populates the variants listed in ADR-0003; Phase 0 reserves
/// the enum shape.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyscallId {
    // Cap core (01..=06)
    CapGrant        = 0x01,
    CapRevoke       = 0x02,
    CapDerive       = 0x03,
    CapPass         = 0x04,
    CapAttestChain  = 0x05,
    CapInspect      = 0x06,
    // Memory (10..=13)
    MemAlloc        = 0x10,
    MemMap          = 0x11,
    MemUnmap        = 0x12,
    MemShare        = 0x13,
    // Process (20..=23)
    ProcSpawn       = 0x20,
    ProcExit        = 0x21,
    ProcYield       = 0x22,
    ProcWait        = 0x23,
    // IPC (30..=34)
    IpcSend         = 0x30,
    IpcRecv         = 0x31,
    IpcCall         = 0x32,
    IpcEndpointNew  = 0x33,
    IpcPoll         = 0x34,
    // Device (40..=42)
    DevMmioMap      = 0x40,
    DevIrqWait      = 0x41,
    DevIrqAck       = 0x42,
    // Store (50..=51)
    StoreOpen       = 0x50,
    StoreStat       = 0x51,
    // Time (60)
    TimeNow         = 0x60,
}

#[cfg(feature = "userspace")]
pub mod userspace {
    //! Userspace syscall issuer. Produces the `svc` instruction and
    //! marshals arguments. Phase 1 fills in per-arch impls.
}
