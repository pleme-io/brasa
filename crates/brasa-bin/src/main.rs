//! # brasa-bin — Phase 0 bootable kernel image
//!
//! Minimum viable brasa kernel: boots on aarch64 QEMU virt, writes a banner
//! to the PL011 UART at `0x0900_0000`, halts.
//!
//! This is a deliberately tiny PoC — Phase 0 exit criterion in the roadmap
//! calls for *"`tronco` boots under QEMU aarch64, prints a typed message
//! via a single syscall, exits cleanly."* Phase 0 has no syscalls yet, so
//! the "typed message" here is a literal `&'static str` rendered through
//! a typed UART writer.
//!
//! Phase 1 will replace the direct PL011 poke with:
//! - `galho-virtio-console` spoken over `seiva::Endpoint<VirtioConsole>`
//! - a real `folha::rt`-linked userspace process doing the printing
//! - an actual syscall path (`ipc_send`) through `casca::Casca`.
//!
//! Until then: raw MMIO, no syscalls, no userspace. The value is proving
//! the toolchain, linker, and boot-drop address are correctly wired.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::write_volatile;

/// QEMU virt machine: PL011 UART0 MMIO base.
/// See QEMU docs: `hw/arm/virt.c`, the `VIRT_UART` memory map entry.
const PL011_DR: *mut u8 = 0x0900_0000 as *mut u8;

/// Very small typed writer. `print` is deliberately infallible (writes are
/// fire-and-forget on PL011); if the UART is broken, we can't tell the user
/// anyway, so we don't try.
struct Uart;

impl Uart {
    #[inline]
    fn write_byte(b: u8) {
        // SAFETY: PL011_DR is the device MMIO register for UART0 on the
        // QEMU virt machine. Writing a byte produces a serial-out event.
        // No alignment concerns (byte write); no aliasing concerns (device
        // memory; no Rust reference aliases).
        unsafe { write_volatile(PL011_DR, b) };
    }

    fn puts(s: &str) {
        for &b in s.as_bytes() {
            Self::write_byte(b);
        }
    }
}

// Entry point. Qemu's `-kernel` flag drops us at the kernel's load address
// (0x4008_0000 for aarch64 virt, per linker script below). At entry:
// - We own the boot core (CPU 0).
// - Cache / MMU state is per firmware defaults.
// - SP is unset; we set it first.
// - Registers x0..x3 hold DTB / boot info; we ignore in Phase 0.
core::arch::global_asm!(
    r#"
.section .text.boot, "ax"
.global _start
_start:
    // (Debug-only first-signal-of-life probe lived here during bring-up:
    //   mov x9, #0x09000000 ; mov w10, #'B' ; strb w10, [x9]
    // Reinstate it the next time we break kmain.)

    // Stack grows down from _stack_top defined in the linker script.
    adrp    x30, _stack_top
    add     x30, x30, :lo12:_stack_top
    mov     sp, x30

    // Zero the .bss region. _bss_start / _bss_end come from the linker.
    adrp    x0, __bss_start
    add     x0, x0, :lo12:__bss_start
    adrp    x1, __bss_end
    add     x1, x1, :lo12:__bss_end
1:
    cmp     x0, x1
    b.ge    2f
    str     xzr, [x0], #8
    b       1b
2:
    bl      kmain
    // kmain doesn't return; if it does, halt.
3:
    wfe
    b       3b
    "#
);

/// Kernel main — Phase 0 PoC body.
#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    Uart::puts("\r\n");
    Uart::puts("======================================\r\n");
    Uart::puts(" brasa — ember kindled\r\n");
    Uart::puts("======================================\r\n");
    Uart::puts(" phase:  ");
    Uart::puts(tronco::PHASE);
    Uart::puts("\r\n arch:   aarch64-unknown-none\r\n");
    Uart::puts(" host:   QEMU virt\r\n");
    Uart::puts(" target: <nothing yet — halting>\r\n");
    Uart::puts("======================================\r\n");
    Uart::puts("\r\n");

    halt()
}

/// Spin halt. Uses `wfe` so QEMU reports the core as idle (no busy-loop).
fn halt() -> ! {
    loop {
        // SAFETY: WFE is always safe; it yields until an event arrives.
        unsafe { core::arch::asm!("wfe") };
    }
}

/// Panic handler — Phase 0 prints the panic location to UART and halts.
/// Phase 1+ will route through the proper typed panic protocol over
/// `seiva::Endpoint<SystemConsole>`.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    Uart::puts("\r\n!! brasa panic !!\r\n  location: ");
    if let Some(loc) = info.location() {
        Uart::puts(loc.file());
        Uart::puts(":");
        // Minimal u32 -> decimal render — we don't pull in fmt here.
        let mut buf = [0u8; 12];
        let s = u32_to_dec(loc.line(), &mut buf);
        Uart::puts(s);
    } else {
        Uart::puts("<unknown>");
    }
    Uart::puts("\r\n");
    halt()
}

fn u32_to_dec(mut n: u32, buf: &mut [u8; 12]) -> &str {
    let mut i = buf.len();
    if n == 0 {
        i -= 1;
        buf[i] = b'0';
    } else {
        while n > 0 {
            i -= 1;
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
    }
    // SAFETY: we only wrote ASCII digits.
    unsafe { core::str::from_utf8_unchecked(&buf[i..]) }
}
