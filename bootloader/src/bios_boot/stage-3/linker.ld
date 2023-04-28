ENTRY(_start)

SECTIONS {
    . = 0x00300000;

    .start : {
        *(.start)
    }
    .text : {
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .data : {
        *(.data .data.*)
    }
    .bss : {
        *(.bss .bss.*)
    }

    . = ALIGN(512);

    magic : {
        SHORT(0x1254)
    }
}