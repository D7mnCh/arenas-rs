use crate::log::log_arena_build;
use std::alloc::{alloc_zeroed, dealloc, Layout};

#[derive(Debug)]
pub struct Arena {
    pub start: *mut u8,
    pub tracker: *mut u8,
    pub used_bytes: usize,
    pub layout: Layout,
}

impl Arena {
    pub fn build(size: usize) -> Self {
        // there's no need for alignment to be more then 1, only if the
        // programmer want more then one arena instance
        const ALIGNMENT: usize = 1;

        let layout = Layout::from_size_align(size, ALIGNMENT).expect(
            "requested length passed isize::MAX boundry \
            and wrapped to negative value",
        );

        let start_ptr = unsafe { alloc_zeroed(layout) };
        log_arena_build(&layout, start_ptr);

        Self {
            tracker: start_ptr,
            start: start_ptr,
            used_bytes: 0,
            layout,
        }
    }

    pub fn bytes_to_push(&self, layout: &Layout) -> usize {
        let requested_bytes = layout.size();
        let padding = self.tracker.align_offset(layout.align());
        let bytes = padding + requested_bytes;

        bytes
    }

    pub fn clear(&mut self) {
        unsafe { dealloc(self.start, self.layout) };
    }
}
