ENTRY(_start)

SECTIONS {
    . = 0x00200000;

    .boot :
    {
        *(.multiboot_header)
    }
    
    .stack : ALIGN(8) {
        *(.stack .stack.*)
    }

    .start : ALIGN(4K) {
        *(.start .start.*)
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
    .eh_frame : {
        *(.eh_frame .eh_frame.*)
    }
    .eh_frame_hdr : {
        *(.eh_frame_hdr .eh_frame_hdr.*)
    }
    .font : {
        *(.font .font.*)
    }

    . = ALIGN(4096);
}
