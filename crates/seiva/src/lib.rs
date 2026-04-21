//! # seiva — brasa typed IPC
//!
//! The sap. Messages, endpoints, and the async wait primitive.
//!
//! An [`EndpointCap<P>`][raiz::CapType] (declared in `raiz` but parameterized by
//! a protocol defined here) carries a typed protocol `P: Protocol`. The
//! type system enforces which messages can be sent and received, and
//! whether cap-passing is part of the message.
//!
//! See [`docs/architecture.md`](../../../docs/architecture.md) and
//! [`ADR-0003`](../../../docs/adrs/0003-syscall-surface.md) for how seiva
//! composes with the syscall surface.

#![cfg_attr(not(feature = "std"), no_std)]

/// A typed IPC protocol. Each endpoint carries exactly one protocol; the
/// type system rejects sending a message not declared in the protocol.
pub trait Protocol: sealed::Sealed {
    /// Messages that can be sent on this endpoint.
    type Message;
    /// Messages that can be received on this endpoint.
    type Reply;
}

/// Phase 1 protocols — virtio-console, virtio-net-device, virtio-blk-device —
/// will be declared here and in their respective `galho-*` driver crates.

pub mod sealed {
    pub trait Sealed {}
}
