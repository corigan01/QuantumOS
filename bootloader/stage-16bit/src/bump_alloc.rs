pub struct BumpAlloc {
    current_ptr: *mut u8,
    end: *mut u8,
}

impl BumpAlloc {
    pub unsafe fn new(current_ptr: *mut u8, size: usize) -> Self {
        Self {
            current_ptr,
            end: current_ptr.add(size),
        }
    }

    pub unsafe fn allocate<'a>(&'a mut self, size: usize) -> Option<&'a mut [u8]> {
        let bumped_ptr = self.current_ptr.add(size);
        if bumped_ptr > self.end {
            return None;
        }

        let allocation_start = self.current_ptr;
        self.current_ptr = bumped_ptr;

        Some(core::slice::from_raw_parts_mut(allocation_start, size))
    }
}
