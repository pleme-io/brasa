//! Syscall dispatch. Implements the [`casca::Casca`] trait kernel-side.
//!
//! Phase 1: the 25 syscalls listed in ADR-0003. Dispatch via a static
//! `[Option<fn>; 256]` table indexed by `casca::SyscallId`.
