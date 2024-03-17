#[repr(packed, C)]
struct DiskAccessPacket {
    packet_size: u8,
    always_zero: u8,
    sectors: u16,
    base_ptr: u16,
    base_segment: u16,
    lba: u64,
}

impl DiskAccessPacket {}
