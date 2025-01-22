ENTRY(_start)

SECTIONS {
    . = 0x100000000000;

    .start : {
        *(.start .start.*)
    }
    . = 0x100000001000;
    .stupid_data : {
        LONG(0)
        LONG(0)
        LONG(0)
        LONG(0)
    }
    
    . = ALIGN(4K);

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
}
