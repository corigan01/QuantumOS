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

    _stage_1_start = .;

    .GDT : {
        *(.GDT .GDT.*)
    }
    .text : {
        *(.text .text.*)
    }
    .data : {
        *(.data .data.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
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

    _stage_1_end = .;

    _stage_1_bytes = _stage_1_end - _stage_1_start;
    _stage_1_sectors = (_stage_1_bytes / 512);
}