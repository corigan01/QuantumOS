use crate::tiny_panic::fail;

#[repr(packed, C)]
pub struct MbrEntry {
    pub bootable: u8,
    _chs: [u8; 7],
    pub lba: u32,
    pub count: u32,
}

pub const PARTITION_PTR: *mut MbrEntry = 0x7BBE as *const MbrEntry as *mut MbrEntry;

pub unsafe fn find_bootable() -> *mut MbrEntry {
    for offset in 0..4 {
        let new_ptr = PARTITION_PTR.add(offset);

        if (&*new_ptr).bootable == 0x80 {
            return new_ptr;
        }
    }

    fail(b'B');
}
