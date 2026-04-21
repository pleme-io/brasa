/*
 * brasa-bin linker script — aarch64 QEMU virt machine.
 *
 * QEMU's `-kernel` flag loads the ELF/binary at the DRAM base + 0x80000
 * (per ARM "no-firmware" convention); DRAM on `virt` starts at 0x40000000.
 * So our entry lands at 0x40080000.
 *
 * Phase 1 will switch to an EFI PE binary loaded by semente via kasou's
 * VZEFIBootLoader; this linker script is for the Phase 0 QEMU-direct path.
 */

ENTRY(_start)

MEMORY {
    RAM (rwx) : ORIGIN = 0x40080000, LENGTH = 128M
}

SECTIONS {
    . = ORIGIN(RAM);

    .text : ALIGN(4K) {
        KEEP(*(.text.boot))
        *(.text .text.*)
    } > RAM

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    } > RAM

    .data : ALIGN(4K) {
        *(.data .data.*)
    } > RAM

    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(8);
        __bss_end = .;
    } > RAM

    .stack : ALIGN(16) {
        . = . + 64K;
        _stack_top = .;
    } > RAM

    /DISCARD/ : {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.ARM.exidx .ARM.exidx.* .ARM.extab .ARM.extab.*)
    }
}
