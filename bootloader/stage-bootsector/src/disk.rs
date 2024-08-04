use crate::tiny_panic::fail;

#[repr(C)]
pub struct DiskAccessPacket {
    packet_size: u8,
    always_zero: u8,
    pub sectors: u16,
    pub base_ptr: u16,
    pub base_segment: u16,
    pub lba: u64,
}

impl DiskAccessPacket {
    pub fn new(sectors: u16, lba: u64, ptr: u32) -> Self {
        let base_segment = (ptr >> 4) as u16;
        let base_ptr = ptr as u16 & 0xF;

        Self {
            packet_size: 0x10,
            always_zero: 0,
            sectors,
            base_ptr,
            base_segment,
            lba,
        }
    }

    #[inline(never)]
    pub fn read(&self, disk: u16) {
        let packet_address = self as *const Self as u16;
        let status: u16;

        unsafe {
            core::arch::asm!("
                push si
                mov si, {packet:x}
                mov ax, 0x4200
                int 0x13
                jc 3f
                mov {status:x}, 0
                jmp 4f
                3:
                mov {status:x}, 1
                4:
                pop si
            ",
                in("dx") disk,
                packet = in(reg) packet_address,
                status = out(reg) status,
            );
        };

        // If the interrupt failed, we want to abort and tell the user
        if status == 1 {
            fail(b'D');
        }
    }
}
