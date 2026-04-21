# ADR-0006: First target — Apple Silicon laptop via kasou

**Status:** Accepted
**Deciders:** pleme-io
**Date:** 2026-04-20

## Context

A new kernel has to boot somewhere. The choice of first target drives everything: driver priorities, cross-compile setup, bring-up tooling, debugging UX. The wrong first target burns months.

Candidate targets for brasa's first boot:

1. Apple Silicon laptop (the author's daily-driver).
2. Raspberry Pi 5 (cheap, documented, aarch64).
3. x86_64 mini-PC (boring, cheap, unloved arch but fleet-relevant).
4. Any-platform QEMU only (no real hardware until later).

## Decision

**First target: Apple Silicon laptop, running brasa as a guest under the `kasou` hypervisor.** Arch is `aarch64`. Drivers first wave: virtio. Bare-metal Apple Silicon is a later phase.

Kasou (`pleme-io/kasou`) is the pleme-io Rust binding for Apple Virtualization.framework. It currently supports `VZLinuxBootLoader` for Linux guests. We will contribute `VZEFIBootLoader` support so kasou can host brasa (and any other EFI-bootable kernel).

### Why VM-first

- **No driver debt on day one.** virtio-net, virtio-blk, virtio-console, virtio-9p, virtio-gpu are all standard, well-documented, and already tested under Apple Virtualization. We do not write an Apple Silicon NVMe driver to get to a prompt; we write virtio-blk.
- **Fast iteration.** `nix build .#brasa-image && nix run .#brasa-kasou` is the edit-compile-test loop. Boot time in kasou is < 2 seconds.
- **Deterministic bring-up.** VM environments are the same every boot. Real hardware introduces UEFI firmware variance, ACPI tables, device tree quirks — all debugging time not spent on the kernel.
- **The author daily-drives an Apple Silicon MacBook.** The development machine is the target machine. No cross-shipping, no separate test hardware.
- **Kasou is already ours.** We control it. Extending it to support brasa is a small, focused change.

### Why Apple Silicon specifically

- It's what's in the laptop. Vertical integration.
- aarch64 is the architecture. Writing kernel code for the same arch that hosts the VM reduces mental overhead.
- Apple Silicon Macs are plentiful in the org. Multiple developers can run brasa locally.
- Eventually we want bare-metal Apple Silicon, and Asahi Linux has done a lot of the GPU/hardware work. Staying aarch64 from the start means no arch migration when we go bare metal.

### Why not Raspberry Pi 5

- Slower. The Pi 5 is a fine machine but a worse iteration platform.
- We'd need physical hardware each developer. Apple Silicon is already on every laptop.
- Pi bring-up would teach us Pi-specific stuff (bcmgenet Ethernet, VideoCore GPU) we don't need in the fleet.

Pi 5 stays a **Phase 3 candidate** as a secondary bare-metal bring-up target, especially if Apple Silicon bare-metal proves too hostile.

### Why not x86_64

- Long-term we want x86_64 support for fleet coverage. But first bring-up on x86_64 would mean writing the kernel for an arch none of us daily-drive, on hardware none of us are emotionally invested in. Velocity penalty.
- aarch64-first is easier; x86_64 is a Phase 6 addition.

### Why not QEMU-only

- QEMU aarch64 is fine and we will use it for CI. But daily-development on QEMU is worse than daily-development on kasou:
  - Kasou uses the real Apple Virtualization framework — close to the hardware reality we'll eventually face.
  - QEMU's aarch64 virt machine is a different device-tree shape than the Apple-hypervisor guest.
  - We have kasou already; QEMU would be a parallel codebase for the host side.

QEMU is the **CI target** (we test every PR in it) but **not the primary development environment**.

### What kasou needs

Kasou currently exposes `VZLinuxBootLoader` (kernel, initrd, cmdline). We need `VZEFIBootLoader` (EFI variable store + an EFI-bootable disk image). Apple's Virtualization.framework supports both; the `objc2-virtualization` bindings expose both.

Estimated effort to add `VZEFIBootLoader` support to kasou: 2 weeks, one PR. Involves:

- New `BootConfig::Efi { disk: PathBuf, variable_store: Option<PathBuf> }` variant in `kasou::config`.
- Configuration wiring through `VmConfig` → `VZVirtualMachineConfiguration`.
- Tests in `kasou` with a hello-world EFI binary.

This work lands in `pleme-io/kasou` as a normal PR before brasa Phase 1 completes.

## Alternatives considered

### Skip VM-first, go straight to bare metal

Rejected. The bring-up cost is ~6 months just to reach a prompt. The risk is too high for Phase 1.

### Use an existing hypervisor (QEMU, Hyper-V, VirtualBox, cloud-hypervisor)

- QEMU: CI-yes, daily-no (see above).
- Hyper-V / VirtualBox: not on Apple Silicon.
- cloud-hypervisor: Rust, clean, but Linux-host only. We're Apple-host.

Kasou matches the host arch + is ours + uses Apple's supported framework. Clear winner.

### Use an Apple UTM or Parallels

Rejected. Closed-source, non-scriptable, not integrable with our Nix build pipeline.

## Consequences

### Good

- Phase 1 Plan is concrete: land `VZEFIBootLoader` in kasou, then bring up brasa under it.
- Developer ergonomics excellent from day one: every laptop is a test machine.
- Everything we learn about Apple Silicon in the VM applies to bare-metal later.

### Bad

- We defer bare-metal driver work to Phase 3. Risk: we discover Apple Silicon bare-metal is much harder than anticipated, and we've built a VM-only OS. Mitigation: Phase 3 keeps Pi 5 as a fallback target; we're never stuck on Apple.
- We depend on Apple's Virtualization.framework continuing to support `VZEFIBootLoader`. Low risk — Apple uses the same mechanism for their own VM products.
- Host requirement is macOS. A Linux developer cannot run brasa via kasou. They can run it under QEMU (also supported). Partial mitigation.

### Neutral

- We're effectively writing two host integrations (kasou + QEMU) from Phase 1. Cost: a few hundred lines of duplicated host-side wiring. Benefit: robustness, CI coverage, developer flexibility.

## Verification

1. By end of Phase 0: `nix run .#brasa-kasou` boots a minimal brasa image and prints a measured banner (hash in dmesg).
2. By end of Phase 1: `nix run .#brasa-kasou` boots a two-process system with typed IPC.
3. Kasou `VZEFIBootLoader` PR merged before brasa Phase 1 starts Phase 1.5 milestone.
4. CI matrix: every PR runs `brasa-qemu-smoke-test` on aarch64.

## Amendments

None yet. Expected amendment: Phase 3 adds "first bare-metal target" which will be a separate ADR.
