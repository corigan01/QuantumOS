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

    pub unsafe fn allocate(&mut self, size: usize) -> Option<&'static mut [u8]> {
        let bumped_ptr = self.current_ptr.add(size);
        if bumped_ptr > self.end {
            return None;
        }

        let allocation_start = self.current_ptr;
        self.current_ptr = bumped_ptr;

        Some(core::slice::from_raw_parts_mut(allocation_start, size))
    }

    pub fn push_ptr_to(&mut self, new_ptr: *mut u8) {
        if new_ptr > self.end {
            panic!("Cannot push ptr past end of allocation area!");
        }

        self.current_ptr = new_ptr;
    }
}
