#![allow(dead_code)]
// NOTE i don't know how to use that arena -.-
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr,
};

struct BumbAlloc {
    start: *mut u8,
    tracker: *mut u8,
    offset: usize,
    // the layout is our region/arena
    layout: Layout,
}

impl BumbAlloc {
    // we need "const generic" cuz an array will be build with a provided
    //length array type must be know at compile time
    fn build_with(length: usize) -> Self {
        let length = length / size_of::<u64>();
        let alignment = 64;

        let layout = Layout::from_size_align(length, alignment).unwrap();
        let ptr = unsafe { alloc_zeroed(layout) };

        Self {
            tracker: ptr,
            start: ptr,
            offset: 0,
            layout,
        }
    }

    fn append_with(&mut self, requested_bytes: usize) -> Option<*mut u8> {
        if self.layout.size() >= self.offset + requested_bytes {
            //todo!("give that pointer an initialized data");
            let ptr = Some(self.tracker);
            self.offset += requested_bytes;
            unsafe {
                self.tracker = self.tracker.add(self.offset);
            }

            let remaining = self.layout.size().saturating_sub(self.offset);
            println!("space remaining for allocation: {remaining}");
            return ptr;
        }

        eprintln!(
            "[ERROR] arena is full :
                 arena provided length               = {length}
                 space remaining for more allocation = {remaining}
            ",
            length = self.layout.size(),
            remaining = 0
        );

        None
    }

    // TODO
    pub fn reallocate_with_new(&mut self, _layout: Layout) {}

    fn uninitialize(&mut self) {
        self.offset = 0;
        self.layout = Layout::new::<()>();
        self.start = ptr::null_mut();
    }

    pub fn bumb(&mut self) {
        unsafe { dealloc(self.start, self.layout) };
        self.uninitialize();
    }
}

#[test]
fn arena_size() {
    const LENGTH: usize = 1000;
    let arena = BumbAlloc::build_with(LENGTH);

    assert_eq!(arena.layout.size(), LENGTH);
}

#[test]
fn appending_arena() {
    const LENGTH: usize = 1000;
    let mut arena = BumbAlloc::build_with(LENGTH);

    let num_bytes = 500;
    arena.append_with(num_bytes);

    assert_eq!(arena.offset, num_bytes);

    arena.append_with(num_bytes);
    // error: can't allocate new memoroy, arena.size < arena.offset
    //arena.append_with(num_bytes);
}

#[test]
fn bumbing_arena() {
    const LENGTH: usize = 548;
    let mut arena = BumbAlloc::build_with(LENGTH);

    let num_bytes = 500;
    arena.append_with(num_bytes);

    arena.bumb();
    assert!(arena.offset != num_bytes);
    assert!(arena.layout.size() == 0);
}

#[test]
fn append_rust_data_to_arena() {
    // NOTE who to allocate that inside my allocator ?
    let array = ['1', '2', '3'];

    const LENGTH: usize = 1000;
    let mut arena = BumbAlloc::build_with(LENGTH);
    if let Some(ptr) = arena.append_with(size_of_val(&array)) {
        //unsafe { ptr.write }
    }

    assert_eq!(arena.offset, 8);
}
