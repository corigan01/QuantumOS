pub struct Inode {
    name: [u8; 11],
    attributes: u8,
    reserved: u8,
    time_tenth: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    cluster_high: u16,
    modified_time: u16,
    modified_date: u16,
    cluster_low: u16,
    file_size: u32,
}
