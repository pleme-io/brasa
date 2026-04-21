# brasa-bin — the bootable kernel image

Phase 0 PoC. A minimal aarch64 binary that boots on QEMU virt, prints a
banner to the PL011 UART, and halts.

Exists to prove the toolchain, target triple, linker script, and boot-drop
address wire up correctly before we invest in real kernel code.

## Build

```bash
cargo build -p brasa-bin --target aarch64-unknown-none --release
```

Produces `target/aarch64-unknown-none/release/brasa-kernel` (ELF).

## Run under QEMU

```bash
qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -nographic \
    -kernel target/aarch64-unknown-none/release/brasa-kernel
```

Or via the nix flake app:

```bash
nix run .#brasa-qemu
```

You should see:

```
======================================
 brasa — ember kindled
======================================
 phase:  Phase 0 — Design
 arch:   aarch64-unknown-none
 host:   QEMU virt
 target: <nothing yet — halting>
======================================
```

Exit QEMU with `Ctrl-A x`.

## Phase 0 → Phase 1 delta

| Phase 0 (this binary) | Phase 1 target |
|---|---|
| raw PL011 MMIO for output | `seiva::Endpoint<VirtioConsole>` message |
| no syscalls | ~25 syscalls per [ADR-0003](../../docs/adrs/0003-syscall-surface.md) |
| no userspace | `folha::rt`-linked userspace process |
| `global_asm!` entry | same, plus proper exception vector table |
| no MMU | 4-level page tables in `tronco::mm` |
| no attestation | `semente` hash + chain extension in `raizame` |
