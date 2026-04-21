//! Per-arch CPU primitives.
//!
//! Phase 0: skeleton. Phase 1: `aarch64` module with MMU setup, exception
//! vectors, timer, `svc` entry. Phase 6: `x86_64` mirror.

#[cfg(target_arch = "aarch64")]
pub mod aarch64 {
    //! aarch64 entry points and CPU primitives.
}

#[cfg(target_arch = "x86_64")]
pub mod x86_64 {
    //! x86_64 entry points and CPU primitives. Phase 6.
}
