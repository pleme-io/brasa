//! # folha — brasa process abstraction and userspace runtime
//!
//! Leaves. The runtime shape of a userspace program: entry point ABI, TLS
//! layout, signal-free exception model, cap bag format. Shared between
//! the kernel (which spawns processes) and userspace startup code.
//!
//! `folha::rt` (feature `rt`) replaces libc for Rust programs. A program
//! that links against `folha::rt` gets a minimal panic handler, allocator,
//! and startup routine that extracts the initial cap bag from the spawn
//! message.
//!
//! See [`docs/architecture.md`](../../../docs/architecture.md).

#![cfg_attr(not(feature = "std"), no_std)]

/// Shape of the initial cap bag delivered at process spawn.
///
/// The kernel fills this out from the `(defservice …)` manifest and the
/// parent's `proc_spawn` call. The child extracts it at startup via
/// [`rt::cap_bag`] and distributes caps to its own services.
#[repr(C)]
pub struct CapBag {
    /// Count of caps in the bag.
    pub count: u32,
    /// Pointer to the cap array; stable for the lifetime of the process.
    pub caps: *const raiz::CapId,
}

#[cfg(feature = "rt")]
pub mod rt {
    //! Userspace runtime — entry point, allocator, panic handler.
    //!
    //! Phase 1: a Rust program can `#[no_main]`, define `fn brasa_main()`,
    //! and link against `folha::rt` to get a working startup routine
    //! that hands it a `CapBag`.
}
