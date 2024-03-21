use core::ptr;

use crate::{
    bios_println,
    io::{Read, Seek},
};

const MAX_SECTORS_PER_READ: u16 = 32;

#[link_section = ".buffer"]
static mut TEMP_BUFFER: [u8; 512] = [0u8; 512];

pub struct BiosDisk {
    id: u16,
    seek: u64,
}

impl BiosDisk {
    pub fn new(disk_id: u16) -> Self {
        Self {
            id: disk_id,
            seek: 0,
        }
    }
}

impl Seek for BiosDisk {
    fn seek(&mut self, pos: u64) -> u64 {
        self.seek = pos;

        pos
    }

    fn stream_position(&mut self) -> u64 {
        self.seek
    }
}

impl Read for BiosDisk {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut reading_start = self.seek;
        let mut reading_end = reading_start + (buf.len() as u64);

        let mut starting_sector = reading_start / 512;
        let ending_sector = reading_end / 512;

        let mut buf_ptr = buf.as_mut_ptr() as u32;

        // Not aligned start
        let non_alignment_start = reading_start & 0x1FF;
        if non_alignment_start != 0 {
            bios_println!("Non alignment start: {}", non_alignment_start);
            DiskAccessPacket::new(1, starting_sector, unsafe { TEMP_BUFFER.as_mut_ptr() }
                as u32)
            .read(self.id);

            unsafe {
                ptr::copy_nonoverlapping(
                    TEMP_BUFFER.as_ptr().add(non_alignment_start as usize),
                    buf.as_mut_ptr(),
                    (512 - non_alignment_start) as usize,
                )
            };

            starting_sector += 1;
            reading_start += non_alignment_start;
            buf_ptr += non_alignment_start as u32;
        }

        // not aligned end
        let non_alignment_end = reading_end & 0x1FF;
        if non_alignment_end != 0 {
            bios_println!("Non alignment end: {}", non_alignment_end);
            DiskAccessPacket::new(1, ending_sector + 1, unsafe {
                TEMP_BUFFER.as_mut_ptr().add(non_alignment_end as usize)
            } as u32)
            .read(self.id);

            unsafe {
                ptr::copy_nonoverlapping(
                    TEMP_BUFFER.as_ptr(),
                    buf.as_mut_ptr()
                        .add((reading_end - non_alignment_end) as usize),
                    (512 - non_alignment_end) as usize,
                );
            }

            reading_end -= non_alignment_end;
        }

        assert!(reading_start % 512 == 0, "Reading Start should be aligned");
        assert!(reading_end % 512 == 0, "Reading End should be aligned");

        while starting_sector != ending_sector {
            let sectors_to_read =
                ((starting_sector - ending_sector) as u16).min(MAX_SECTORS_PER_READ);
            DiskAccessPacket::new(sectors_to_read, starting_sector, buf_ptr).read(self.id);

            starting_sector += sectors_to_read as u64;
            buf_ptr += sectors_to_read as u32 * 512;
        }

        todo!("{:x?}", buf)
    }
}

#[derive(Debug)]
#[repr(C)]
struct DiskAccessPacket {
    packet_size: u8,
    always_zero: u8,
    sectors: u16,
    base_ptr: u16,
    base_segment: u16,
    lba: u64,
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
                jc 1f
                mov {status:x}, 0
                jmp 2f
                1:
                mov {status:x}, 1
                2:
                pop si
            ",
                in("dx") disk,
                packet = in(reg) packet_address,
                status = out(reg) status,
            );
        };

        // If the interrupt failed, we want to abort and tell the user
        if status == 1 {
            panic!(
                "Failed to read from disk!\n{:#?}\nPTR={:x}",
                self,
                (self.base_segment as u32 * 16) + (self.base_ptr as u32)
            );
        }
    }
}
