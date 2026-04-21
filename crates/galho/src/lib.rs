//! # galho — brasa driver framework
//!
//! Branches. Types that drivers share across the PCI/USB/virtio/custom
//! device matrix. Individual drivers live in separate repos
//! (`pleme-io/galho-virtio-net`, `galho-virtio-blk`, …) and depend on
//! this crate + `casca` + `seiva`.
//!
//! See [`docs/architecture.md`](../../../docs/architecture.md) and
//! [`ADR-0004`](../../../docs/adrs/0004-tatara-lisp-authoring.md) for the
//! `(defdriver …)` authoring surface.

#![cfg_attr(not(feature = "std"), no_std)]

/// Generic driver lifecycle. Implemented by each driver crate.
///
/// Authored in tatara-lisp as `(defdriver …)` and generated into an
/// `impl Driver for MyDriver { … }` by `forja` during compilation.
pub trait Driver {
    /// One-shot initialization. Called once after cap-bag delivery.
    fn init(&mut self) -> Result<(), DriverError>;

    /// Attach to a specific device instance (from enumeration).
    fn attach(&mut self, device: DeviceHandle) -> Result<(), DriverError>;

    /// Detach cleanly. Returns caps to the parent.
    fn detach(&mut self) -> Result<(), DriverError>;
}

/// Opaque handle to a specific enumerated device. Composition of device-bus
/// identifiers (PCI BDF, USB address, virtio queue selector).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceHandle(pub u64);

/// Driver error taxonomy. Phase 1 expansion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriverError {
    CapMissing,
    DeviceAbsent,
    ProtocolViolation,
    Timeout,
    Unsupported,
}

pub mod pci {
    //! PCI enumeration + matching. Phase 1.
}

pub mod virtio {
    //! virtio-common types shared across virtio-net / virtio-blk / etc. Phase 1.
}
