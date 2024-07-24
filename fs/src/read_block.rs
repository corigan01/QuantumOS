use crate::error::Result;

/// # Block Device
/// A device that can only read 'blocks' of bytes at a time.
///
///
/// ### Example
/// For example, sometimes you have drives that can only read one
/// sector (~512 or so bytes) at a time. However, for filesystems
/// and other logic, this can be hard to program for. This trait
/// implementes reading from such block devices.
pub trait BlockDevice {
    /// # Block Size
    /// The size of each of the blocks this media can read.
    const BLOCK_SIZE: usize;

    /// # Read Device
    /// Read one block from the device given the block's offset.
    ///
    /// # Note
    /// Since the size of each block is not known at compile time,
    /// it must be up to the programmer to keep track of providing
    /// the bytes this block device had read.
    fn read_block<'a>(&'a mut self, block_offset: usize) -> Result<&'a [u8]>;
}

pub fn read_smooth_from_block_device<Device: BlockDevice>(
    device: &mut Device,
    offset_bytes: u64,
    data: &mut [u8],
) -> Result<usize> {
    let mut data_copied = 0;

    loop {
        let block_index =
            ((offset_bytes + data_copied as u64) / Device::BLOCK_SIZE as u64) as usize;
        let block_offset =
            ((offset_bytes + data_copied as u64) % Device::BLOCK_SIZE as u64) as usize;

        let index_begin = data_copied;
        let index_end =
            (Device::BLOCK_SIZE - block_offset).min(data.len() - data_copied) + index_begin;

        assert!(
            index_begin <= index_end,
            "Index end ({index_end}) should always follow index begin ({index_begin})!"
        );

        // Done Reading
        if index_begin == index_end {
            break;
        }

        let device_block = device.read_block(block_index)?;
        let reading_bytes = index_end - index_begin;

        data[index_begin..index_end]
            .copy_from_slice(&device_block[block_offset..(block_offset + reading_bytes)]);

        data_copied += reading_bytes;
    }

    Ok(data.len())
}

#[cfg(test)]
mod test {
    use super::{read_smooth_from_block_device, BlockDevice};

    struct Dummy {
        buf: [u8; 10],
    }

    impl Dummy {
        pub fn new() -> Self {
            Self { buf: [0; 10] }
        }
    }

    impl BlockDevice for Dummy {
        const BLOCK_SIZE: usize = 10;

        fn read_block<'a>(&'a mut self, block_offset: usize) -> crate::error::Result<&'a [u8]> {
            self.buf = [block_offset as u8; 10];
            Ok(&self.buf)
        }
    }

    #[test]
    fn test_reading_first_block() {
        let mut dummy = Dummy::new();
        assert_eq!(
            dummy.read_block(0).unwrap(),
            [0; 10],
            "Reading Block should return correct bytes"
        );
    }

    #[test]
    fn test_reading_nth_block() {
        let mut dummy = Dummy::new();

        for block_offset in 0..10 {
            assert_eq!(
                dummy.read_block(block_offset).unwrap(),
                &[block_offset as u8; 10],
                "Reading Block should return correct bytes"
            );
        }
    }

    #[test]
    fn test_smooth_reading_first_whole() {
        let mut dummy = Dummy::new();

        let mut target = [255; 10];
        read_smooth_from_block_device(&mut dummy, 0, &mut target).unwrap();

        assert_eq!(
            target, [0; 10],
            "Expected smooth reading to be one 'sector' only!"
        );
    }

    #[test]
    fn test_smooth_reading_nth_whole() {
        let mut dummy = Dummy::new();

        let mut target = [255; 10];

        for nth in 0..10 {
            read_smooth_from_block_device(&mut dummy, nth * 10, &mut target).unwrap();

            assert_eq!(
                target, [nth as u8; 10],
                "Expected smooth reading to be one 'sector' only!"
            );
        }
    }

    #[test]
    fn test_smooth_over_boarder_two_whole() {
        let mut dummy = Dummy::new();

        let mut target = [255; 20];

        read_smooth_from_block_device(&mut dummy, 0, &mut target).unwrap();

        assert_eq!(
            target,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            "Expected bytes to switch when reading different sector"
        );
    }

    #[test]
    fn test_smooth_over_boarder_two_offset() {
        let mut dummy = Dummy::new();

        let mut target = [255; 20];

        read_smooth_from_block_device(&mut dummy, 5, &mut target).unwrap();

        assert_eq!(
            target,
            [0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2],
            "Expected bytes to switch when reading different sector"
        );
    }

    #[test]
    fn test_smooth_over_boarder_three_offset() {
        let mut dummy = Dummy::new();

        let mut target = [255; 30];

        read_smooth_from_block_device(&mut dummy, 5, &mut target).unwrap();

        assert_eq!(
            target,
            [
                0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3,
                3, 3
            ],
            "Expected bytes to switch when reading different sector"
        );
    }
}
