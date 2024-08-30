ENTRY(bios_entry)

MEMORY {
    /* 0x00000000	0x000003FF	1 KiB	Real Mode IVT (Interrupt Vector Table)	unusable in real mode	640 KiB RAM ("Low memory") */
    IVT          (r)   : ORIGIN = 0x00000000, LENGTH = 0x00400

    /* 0x00000400	0x000004FF	256 bytes	BDA (BIOS data area) */
    BDA          (r)   : ORIGIN = 0x00000400, LENGTH = 0x00100

    /* 0x00000500	0x00007BFF	almost 30 KiB	Conventional memory	usable memory */
    LOW_30K      (rwx) : ORIGIN = 0x00000500, LENGTH = 0x076FF

    /* 0x00007C00	0x00007DFF	512 bytes       BootSector */
    BOOT_SEC     (rwx) : ORIGIN = 0x00007C00, LENGTH = 0x001BE
    PART_TABLE   (r)   : ORIGIN = 0x00007DBE, LENGTH = 0x00040
    BOOT_MAGIC   (r)   : ORIGIN = 0x00007DFE, LENGTH = 0x00002

    /* 0x00007E00	0x0007FFFF	480.5 KiB	Conventional memory */
    CODE_LIMIT   (rwx) : ORIGIN = 0x00007E00, LENGTH = 0x08200
    DATA_LIMIT   (rwx) : ORIGIN = 0x00010000, LENGTH = 0x6FFFF

    /* 0x00080000	0x0009FFFF	128 KiB	EBDA (Extended BIOS Data Area)	partially used by the EBDA */
    EBDA (r) : ORIGIN = 0x00080000, LENGTH = 0x1FFFF

    /*0x000A0000	0x000BFFFF	128 KiB	Video display memory	hardware mapped	384 KiB System / Reserved ("Upper Memory")*/
    VIDEO_DISPLAY (rw) : ORIGIN = 0x000A0000, LENGTH = 0x1FFFF

    /*
    0x000C0000	0x000C7FFF	32 KiB (typically)	Video BIOS	ROM and hardware mapped / Shadow RAM
    0x000C8000	0x000EFFFF	160 KiB (typically)	BIOS Expansions
    0x000F0000	0x000FFFFF	64 KiB	Motherboard BIOS
    */
    RESERVED     (r)   : ORIGIN = 0x000C0000, LENGTH = 0x3FFFF
}

SECTIONS {
    .boot : {
        *(.boot .boot.* .loader .loader.* .text .text.* .data .data.* .got .got.*)
    } > BOOT_SEC

    .magic : {
        SHORT(0xaa55)
    } > BOOT_MAGIC
}
