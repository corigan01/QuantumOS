ENTRY(begin)

SECTIONS {
    . = 0x7c00;

    .boot : {
        *(.boot .boot.*)
    }

    . = 0x7c00 + 510;

    .bootable_magic : {
        SHORT(0xAA55)
    }

    . = 0x7E00;

    .data : {
        *(.data .data.*)
    }
    .text : {
        *(.text .text.*)
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

    _stage_1_end_sectors = ((. - 0x7E00) / 512);
}