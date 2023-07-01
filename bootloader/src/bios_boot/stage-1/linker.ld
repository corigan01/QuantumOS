ENTRY(begin)

MEMORY {
    /*
    0x00000000	0x000003FF	1 KiB	Real Mode IVT (Interrupt Vector Table)	unusable in real mode	640 KiB RAM ("Low memory")
    */
    IVT          (r)   : ORIGIN = 0x00000000, LENGTH = 0x00400

    /*
    0x00000400	0x000004FF	256 bytes	BDA (BIOS data area)
    */
    BDA          (r)   : ORIGIN = 0x00000400, LENGTH = 0x00100

    /*
    0x00000500	0x00007BFF	almost 30 KiB	Conventional memory	usable memory
    */
    LOW_30K      (rwx) : ORIGIN = 0x00000500, LENGTH = 0x076FF

    /*
    0x00007C00	0x00007DFF	512 bytes	Your OS BootSector
    */
    BOOT_SEC     (rwx) : ORIGIN = 0x00007C00, LENGTH = 0x001FD
    BOOT_MAGIC   (r)   : ORIGIN = 0x00007DFD, LENGTH = 0x00002

    /*
    0x00007E00	0x0007FFFF	480.5 KiB	Conventional memory
    */
    CODE_LIMIT   (rwx) : ORIGIN = 0x00007E00, LENGTH = 0x08200
    DATA_LIMIT   (rwx) : ORIGIN = 0x00010000, LENGTH = 0x6FFFF

    /*
    0x00080000	0x0009FFFF	128 KiB	EBDA (Extended BIOS Data Area)	partially used by the EBDA
    0x000A0000	0x000BFFFF	128 KiB	Video display memory	hardware mapped	384 KiB System / Reserved ("Upper Memory")
    0x000C0000	0x000C7FFF	32 KiB (typically)	Video BIOS	ROM and hardware mapped / Shadow RAM
    0x000C8000	0x000EFFFF	160 KiB (typically)	BIOS Expansions
    0x000F0000	0x000FFFFF	64 KiB	Motherboard BIOS
    */
    RESERVED     (r)   : ORIGIN = 0x00080000, LENGTH = 0xFFFFF
}

SECTIONS {

    .boot : {
        *(.boot .boot.*)
    } > BOOT_SEC

    .bootable_magic : {
        SHORT(0xAA55)
    } > BOOT_MAGIC

    _stage_1_start = .;

    /*
        Since we are in 16-bit mode here, it is very important that our code can not go over
        this limit! Even once we turn on 'UNREAL_MODE' we must still have the code in the first
        64k block of memory, or the '0'th segment.
    */
    .text : {
        *(.text .text.*)
    } > CODE_LIMIT


    /*
        The rest of the segments can fit in the DATA_LIMIT section because we are wanting to be
        in 'UNREAL_MODE' so we can access memory higher then 64k chunks.
     */
    .GDT : {
        *(.GDT .GDT.*)
    } > DATA_LIMIT
    .data : {
        *(.data .data.*)
    } > DATA_LIMIT
    .rodata : {
        *(.rodata .rodata.*)
    } > DATA_LIMIT
    .bss :  {
        *(.bss .bss.*)
    } > DATA_LIMIT

    /*
        Here we want to calculate how much data we used so we can tell the lil' bootloader script how many sectors
        to load into memory.
    */
    _stage_1_end = .;
    _stage_1_bytes = _stage_1_end - _stage_1_start;
    _stage_1_sectors = (_stage_1_bytes / 512);
}