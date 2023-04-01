ENTRY(begin)

SECTIONS {
    . = 0x500;
    _stack_start = .;
    . = 0x7c00;
    _stack_end = .;

    .boot : {
        *(.boot .boot.*)
    }

    . = 0x7c00 + 510;

    .bootable_magic : {
        SHORT(0xAA55)
    }

    .text : {
        *(.text .text.*)
    }
    .data : {
        *(.data .data.*)
    }
    .bss :  {
        *(.bss .bss.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .eh_frame : {
       *(.eh_frame .eh_frame.*)
    }
    .eh_frame_hdr : {
       *(.eh_frame_hdr .eh_frame_hdr.*)
    }

    _stage_1_end_sectors = ((. - 0x7c00) / 512);
}