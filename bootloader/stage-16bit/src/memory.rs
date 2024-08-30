use bios::memory::MemoryEntry;
use core::mem::MaybeUninit;

#[no_mangle]
static mut MEMORY_MAP_AREA: MaybeUninit<[MemoryEntry; 16]> = MaybeUninit::zeroed();

pub fn memory_map() -> &'static [MemoryEntry] {
    let stable_regions =
        unsafe { bios::memory::read_mapping(MEMORY_MAP_AREA.assume_init_mut()) }.unwrap();
    unsafe { &MEMORY_MAP_AREA.assume_init_mut()[..stable_regions] }
}
