#![allow(dead_code)]
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr,
};

const DATA_ALIGNMENT: usize = 1;

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
            "[INFO] requested {} bytes of memory to allocate an arena",
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
        let data_ptr: *mut u8;
        if self.layout.size() >= self.used_bytes + layout.align() {
            println!("[INFO] requesting {} bytes to allocate", layout.size());
            // return this pointer to current tracker
            data_ptr = self.tracker;
            // updating arena tracker pointer
            let offset = BumbAlloc::align_forward(self.used_bytes, &layout) + layout.size();
            unsafe {
                // offset the pointer(tracker) to get an align address
                println!("[INFO] tracker points to : {:?}", self.tracker);
                self.tracker = self.tracker.add(offset);
                println!("[INFO] offsetting track pointer by {offset}");
                println!("[INFO] tracker now points to : {:?}", self.tracker);
            }

            // updating arena used bytes
            let padding = BumbAlloc::align_forward(self.used_bytes, &layout);
            let new_bytes = layout.size() + padding;
            println!(
                "[INFO] add {new} bytes to arena's used bytes",
                new = new_bytes
            );

            self.used_bytes += new_bytes;
            println!("[INFO] total used bytes : {}", self.used_bytes);
        } else {
            eprintln!("[ERROR] requested allocation is more then arena's remaining space",);
            data_ptr = ptr::null_mut()
        };

        let remaining = self.layout.size().saturating_sub(self.used_bytes);
        println!("[INFO] remaining space: {remaining}\n");

        data_ptr
    }

    fn align_forward(current_used_bytes: usize, layout: &Layout) -> usize {
        let modulo = current_used_bytes % layout.align();
        let mut padding = 0;

        if modulo != 0 {
            padding = layout.align() - modulo;
            println!("[INFO] skip {padding} bytes of padding");
        }

        padding
    }

    // TODO think
    pub fn reallocate_with_new(&mut self, _layout: Layout) {}

    fn uninitialize(&mut self) {
        self.used_bytes = 0;
        self.layout = Layout::new::<()>();
        self.start = ptr::null_mut();
        self.tracker = ptr::null_mut();
    }

    pub fn bumb(&mut self) {
        unsafe { dealloc(self.start, self.layout) };
        self.uninitialize();
    }
}

// TODO add more tests
#[cfg(test)]
mod test {
    const LENGTH: usize = 1000;

    use super::*;
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
    fn not_enough_space_to_allocate() {
        let mut arena = BumbAlloc::build_with(0);
        let layout = Layout::new::<i32>();
        let ptr = arena.append_with(&layout);
        // not enough space, arena.used_bytes will not change
        assert_eq!(arena.used_bytes, 0);
        assert!(ptr.is_null());
    }

    #[test]
    fn bumbing_arena() {
        let mut arena = BumbAlloc::build_with(LENGTH);

        let layout = Layout::new::<i32>();
        let _ = arena.append_with(&layout);

        arena.bumb();

        assert_eq!(arena.layout.size(), 0);
        // can't have alignment of 0 in rust
        assert_eq!(arena.layout.align(), 1);
        assert_eq!(arena.used_bytes, 0);
        assert_eq!(arena.tracker, ptr::null_mut());
        assert_eq!(arena.start, ptr::null_mut());
    }

    #[test]
    fn append_primitives_to_arena() {
        let mut arena = BumbAlloc::build_with(LENGTH);

        let layout = Layout::new::<[i16; 2]>();
        let ptr_1 = arena.append_with(&layout).cast::<[i16; 2]>();
        assert_eq!(arena.used_bytes, 4);

        let layout = Layout::new::<[i64; 3]>();
        let ptr_2 = arena.append_with(&layout).cast::<[i64; 3]>();
        let padding = 4;
        let prev_area_used_bytes = 4;
        assert_eq!(
            arena.used_bytes,
            prev_area_used_bytes + layout.size() + padding
        );

        // testing arena.tracker
        unsafe {
            ptr_1.write([1, 2]);
            ptr_2.write([3, 4, 5]);
            assert_eq!(ptr_1.read(), [1, 2]);
            assert_eq!(ptr_2.read(), [3, 4, 5]);
        }

        arena.bumb();
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
                more_more_data: '1',
            });

            dbg!(ptr_1);
            dbg!(ptr_1.read());
            dbg!(ptr_2);
            dbg!(ptr_2.read());
        }
        arena.bumb();
    }
}
