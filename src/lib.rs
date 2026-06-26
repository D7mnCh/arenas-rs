#![allow(dead_code)]
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr,
};

const DATA_ALIGNMENT: usize = 1;
const LENGTH: usize = 1000;

struct BumbAlloc {
    start: *mut u8,
    tracker: *mut u8,
    used_bytes: usize,
    layout: Layout,
}

impl BumbAlloc {
    // we need "const generic" cuz an array will be build with a provided
    //length array type must be know at compile time
    fn build_with(length: usize) -> Self {
        let layout = Layout::from_size_align(length, DATA_ALIGNMENT).unwrap();
        println!(
            "[INFO] requested {} bytes of memory to allocate",
            layout.size()
        );
        let ptr = unsafe { alloc_zeroed(layout) };

        Self {
            tracker: ptr,
            start: ptr,
            used_bytes: 0,
            layout,
        }
    }

    fn append_with(&mut self, layout: &Layout) -> *mut u8 {
        if self.layout.size() >= self.used_bytes + layout.align() {
            println!("[INFO] requesting {} bytes to allocate", layout.size());
            // return this pointer to current tracker
            let ptr = self.tracker;
            // updating arena tracker pointer
            let to_be_aligned = align_requested_data(self.used_bytes, &layout);
            println!("[INFO] offsetting track pointer by {to_be_aligned}");
            unsafe {
                // offset the pointer(tracker) to get an align address
                self.tracker = self.tracker.add(to_be_aligned);
            }

            // updating arena used bytes
            println!("[INFO] add {to_be_aligned} bytes to arena's used bytes");
            self.used_bytes += align_requested_data(self.used_bytes, &layout);

            let remaining = self.layout.size().saturating_sub(self.used_bytes);
            println!("[INFO] space remaining for allocation: {remaining}");
            return ptr;
        }
        panic!(
            "[ERROR] arena is full :
                 arena provided length               = {length}
                 space remaining for more allocation = {remaining}
            ",
            length = self.layout.size(),
            remaining = 0
        );
    }

    // TODO
    pub fn reallocate_with_new(&mut self, _layout: Layout) {}

    fn uninitialize(&mut self) {
        self.used_bytes = 0;
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
    let arena = BumbAlloc::build_with(LENGTH);

    assert_eq!(arena.layout.size(), LENGTH);
}

#[test]
fn appending_arena() {
    const LENGTH: usize = 10;
    let mut arena = BumbAlloc::build_with(LENGTH);

    let i32_layout = Layout::new::<i32>();
    arena.append_with(&i32_layout);
    arena.append_with(&i32_layout);
    let i16_layout = Layout::new::<i16>();
    arena.append_with(&i16_layout);

    assert_eq!(arena.used_bytes, LENGTH);
}

#[test]
fn bumbing_arena() {
    let mut arena = BumbAlloc::build_with(LENGTH);

    let layout = Layout::new::<i32>();
    arena.append_with(&layout);

    arena.bumb();

    assert!(arena.used_bytes != layout.align());
    assert!(arena.layout.size() == 0);
}

#[test]
fn append_primitives_to_arena() {
    let mut arena = BumbAlloc::build_with(LENGTH);

    let layout = Layout::new::<[i16; 2]>();
    let ptr_1 = arena.append_with(&layout).cast::<[i16; 2]>();
    assert_eq!(arena.used_bytes, 4);

    let layout = Layout::new::<[i32; 3]>();
    let ptr_2 = arena.append_with(&layout).cast::<[i32; 3]>();
    let padding = 4;
    let prev_area_used_bytes = 4;
    assert_eq!(
        arena.used_bytes,
        prev_area_used_bytes + layout.size() + padding
    );

    // testing arena.tracker
    unsafe {
        ptr_1.write([1, 2]);
        ptr_2.write([1000, 3324, 1231]);
        assert!(ptr_1.read() == [1, 2]);
        assert!(ptr_2.read() == [1000, 3324, 1231]);
    }
}

#[test]
fn append_structs_to_arena() {
    #[derive(Debug)]
    struct Foo {
        data: bool,
        more_data: u64,
    }
    #[derive(Debug)]
    struct Boo {
        some_data: u8,
        more_more_data: char,
    }

    let foo_layout = Layout::new::<Foo>();
    let boo_layout = Layout::new::<Boo>();
    let mut arena = BumbAlloc::build_with(LENGTH);

    let ptr_1 = arena.append_with(&foo_layout).cast::<Foo>();
    assert_eq!(arena.used_bytes, foo_layout.size());

    let ptr_2 = arena.append_with(&boo_layout).cast::<Boo>();
    let prev_area_used_bytes = 16;
    assert_eq!(arena.used_bytes, prev_area_used_bytes + boo_layout.size());

    unsafe {
        ptr_1.write(Foo {
            data: true,
            more_data: 2837,
        });

        ptr_2.write(Boo {
            some_data: 255,
            more_more_data: 'h',
        });

        dbg!(ptr_1);
        dbg!(ptr_1.read());
        dbg!(ptr_2);
        dbg!(ptr_2.read());
    }
}

fn align_requested_data(current_used_bytes: usize, layout: &Layout) -> usize {
    let bytes_to_add = if layout.size() != layout.align() {
        let bytes_to_align_reqstd_data =
            if current_used_bytes == 0 || current_used_bytes % layout.align() == 0 {
                0
            } else if current_used_bytes > layout.align() {
                current_used_bytes.div_ceil(layout.align())
            } else {
                layout.align() - current_used_bytes
            };
        bytes_to_align_reqstd_data + layout.size()
    } else {
        if current_used_bytes >= layout.align() {
            let offset = layout.align();
            offset
        } else {
            let offset = layout.align() - current_used_bytes;
            offset
        }
    };
    bytes_to_add
}

//fn align_requested_data(current_bytes_used: usize, layout: &Layout) -> usize {
//if data_alignment % DATA_ALIGNMENT == 0 {
//    return data_alignment;
//}
//else if DATA_ALIGNMENT < data_alignment {
//    println!("[WARNING] requested layout alignment is greater then arena alignemnt");
//    let multiplier = data_alignment.div_ceil(DATA_ALIGNMENT);
//    return DATA_ALIGNMENT * multiplier;
//} else if DATA_ALIGNMENT > data_alignment {
//    println!("[WARNING] requested layout alignment is less then arena alignemnt");
//    return DATA_ALIGNMENT;
//} else {
//    return 0;
//}
//}
