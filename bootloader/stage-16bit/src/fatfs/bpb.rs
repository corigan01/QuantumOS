use super::{FatKind, ReadSeek};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bpb {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    number_fats: u8,
    root_entries: u16,
    total_sectors_fat16: u16,
    media_type: u8,
    fat_sectors_fat16: u16,
    sectors_per_track: u16,
    head_count: u16,
    hidden_sectors: u32,
    total_sectors_fat32: u32,
    extended: ExtendedBpb,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Bpb16 {
    drive_number: u8,
    reserved: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_str: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Bpb32 {
    fat_size: u32,
    ext_flags: u16,
    fat_version: u32,
    root_cluster: u32,
    fs_info: u16,
    boot_sector: u16,
    reserved: [u8; 12],
    drive_number: u8,
    reserved2: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_str: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
union ExtendedBpb {
    fat16: Bpb16,
    fat32: Bpb32,
}

enum ExtendedKind<'a> {
    Fat16(&'a Bpb16),
    Fat32(&'a Bpb32),
}

impl Bpb {
    const ROOT_ENTRY_SIZE: usize = 32;
    const FAT12_CLUSTERS: usize = 4085;
    const FAT16_CLUSTERS: usize = 65525;

    pub(crate) fn new<Disk: ReadSeek>(disk: &mut Disk) -> Result<Self, &'static str> {
        let mut bpb = unsafe { core::mem::zeroed::<Self>() };
        disk.seek(0);

        // Treat ourself as if we were a slice
        let self_slice = unsafe {
            &mut (*core::ptr::slice_from_raw_parts_mut(
                ((&mut bpb) as *mut Self) as *mut u8,
                core::mem::size_of_val(&bpb),
            ))
        };

        disk.read(self_slice);

        // TODO: Add more checks for BPB to ensure that it is valid before returning it
        if bpb.bytes_per_sector == 0 || bpb.sectors_per_cluster == 0 {
            return Err("Not valid BPB structure on disk");
        }

        Ok(bpb)
    }

    fn root_sectors(&self) -> usize {
        // 3.5 Determination of FAT type when mounting the Volume (page: 14)
        ((self.root_entries as usize * Self::ROOT_ENTRY_SIZE)
            + (self.bytes_per_sector as usize - 1))
            / (self.bytes_per_sector as usize)
    }

    pub fn total_sectors(&self) -> usize {
        if self.total_sectors_fat16 != 0 {
            self.total_sectors_fat16 as usize
        } else {
            self.total_sectors_fat32 as usize
        }
    }

    fn fat_sectors(&self) -> usize {
        if self.fat_sectors_fat16 != 0 {
            self.fat_sectors_fat16 as usize
        } else {
            unsafe { self.extended.fat32.fat_size as usize }
        }
    }

    fn clusters(&self) -> usize {
        let data_sectors = self.total_sectors()
            - (self.reserved_sectors as usize
                + (self.number_fats as usize * self.fat_sectors())
                + self.root_sectors());

        data_sectors / (self.sectors_per_cluster as usize)
    }

    pub fn kind(&self) -> FatKind {
        match self.clusters() {
            ..=Self::FAT12_CLUSTERS => FatKind::Fat12,
            ..=Self::FAT16_CLUSTERS => FatKind::Fat16,
            _ => FatKind::Fat32,
        }
    }

    fn safe_extended<'a>(&'a self) -> ExtendedKind<'a> {
        match self.kind() {
            FatKind::Fat12 | FatKind::Fat16 => ExtendedKind::Fat16(unsafe { &self.extended.fat16 }),
            FatKind::Fat32 => ExtendedKind::Fat32(unsafe { &self.extended.fat32 }),
        }
    }
}
