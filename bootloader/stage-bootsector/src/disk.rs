#[repr(packed, C)]
struct DiskAccessPacket {
    packet_size: u8,
    always_zero: u8,
    sectors: u16,
    base_ptr: u16,
    base_segment: u16,
    lba: u64,
}

impl DiskAccessPacket {
    pub fn read(disk: u8, sectors: u16, lba: u64, ptr: (u16, u16)) {
        let base_segment = ptr.0;
        let base_ptr = ptr.1;

        let packet = Self {
            packet_size: core::mem::size_of::<Self>() as u8,
            always_zero: 0,
            sectors,
            base_ptr,
            base_segment,
            lba,
        };

        let packet_address = &packet as *const Self;

        unsafe {
            core::arch::asm!("
                push si
                mov si, {packet:x}
                int 0x13
                push ax
                mov al, 'D'
                jc .fail
                pop ax
                pop si
                pop si
            ",
                packet = in(reg) packet_address,
                in("dl") disk,
                in("ax") 0x4200,
            )
        };
    }
}
