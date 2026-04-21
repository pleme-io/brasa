//! # semente — brasa bootloader
//!
//! The seed. Loads the kernel ELF, measures it into `H₀`, hands off.
//!
//! Phase 0: lib-only placeholder. Phase 1: becomes a proper UEFI application
//! (`[[bin]]` target with a custom linker script, built for the
//! `aarch64-unknown-uefi` target).
//!
//! See [`ADR-0002`](../../../docs/adrs/0002-attestation-chain.md) and
//! [`ADR-0006`](../../../docs/adrs/0006-first-target-apple-silicon-kasou.md).

#![cfg_attr(not(feature = "std"), no_std)]

/// Boot information handed from semente to tronco.
///
/// `tronco`'s entry point receives a pointer to this struct in a
/// well-known register (x0 on aarch64).
#[repr(C)]
pub struct BootInfo {
    /// BLAKE3 hash of the loaded `tronco.elf` — the first link in the
    /// attestation chain.
    pub tronco_hash: [u8; 32],
    /// Physical memory map entries (base, length, kind).
    /// Count given by `memory_map_count`.
    pub memory_map: *const MemoryMapEntry,
    pub memory_map_count: u32,
    /// Pointer to the packed `BootManifest` (produced from `(defsystem …)`).
    pub manifest: *const core::ffi::c_void,
}

/// One entry of the memory map as presented by UEFI firmware. Converted by
/// `tronco::mm` into a typed physical frame allocator state.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub kind: MemoryKind,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryKind {
    Usable = 0,
    Reserved = 1,
    AcpiReclaimable = 2,
    AcpiNvs = 3,
    Mmio = 4,
    BootServicesCode = 5,
    BootServicesData = 6,
    RuntimeServicesCode = 7,
    RuntimeServicesData = 8,
    Kernel = 9,
    Manifest = 10,
}
