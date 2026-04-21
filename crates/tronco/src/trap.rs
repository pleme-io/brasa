//! Exception and interrupt handling, syscall entry, panic handling.
//!
//! Phase 1: aarch64 exception vector table, `svc` decode path, page-fault
//! handler, IRQ dispatch to userspace via [`raiz::IrqCap`].
