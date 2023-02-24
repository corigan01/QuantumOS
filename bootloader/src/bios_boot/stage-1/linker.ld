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

    .boot : {
        *(.boot .boot.*)
    }
    .text : {
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .data : {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }
    .bss :  {
        *(.bss .bss.*)
    }
    .eh_frame : {
       *(.eh_frame .eh_frame.*)
    }
    .eh_frame_hdr : {
       *(.eh_frame_hdr .eh_frame_hdr.*)
    }


    . = ALIGN(512);

    .end_magic : {
        LONG(0xdeadbeef)
    }

    _stage_1_end_sectors = ((. - 0x7c00) / 512);

}