//! # tronco — brasa kernel core
//!
//! The trunk of the tree. Memory management, scheduling, IPC primitives,
//! syscall dispatch, arch-specific entry points. Everything else hangs off
//! the trunk.
//!
//! See [`docs/architecture.md`](../../../docs/architecture.md) and the ADRs
//! under [`docs/adrs/`](../../../docs/adrs/) for the design.
//!
//! # Module map
//!
//! - [`arch`] — per-arch CPU primitives, MMU, exception vectors, timer.
//! - [`mm`]   — physical frame allocator, virtual address spaces, page tables.
//! - [`sched`]— capability-based scheduler; no PIDs.
//! - [`ipc`]  — kernel-side IPC implementation; types live in `seiva`.
//! - [`syscall`] — syscall dispatch; implementations of the `casca::Casca` trait.
//! - [`trap`] — exception/interrupt handling, syscall entry, panic handling.
//!
//! All modules are skeletons at Phase 0. See the roadmap for what each gets
//! filled in with and when.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

pub mod arch;
pub mod ipc;
pub mod mm;
pub mod sched;
pub mod syscall;
pub mod trap;

/// Phase of the kernel bring-up. Present so the `semente` bootloader can
/// print a banner; removed once the boot log is typed properly.
pub const PHASE: &str = "Phase 0 — Design";
