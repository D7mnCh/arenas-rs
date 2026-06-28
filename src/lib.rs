#![allow(dead_code)]
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr,
};

const ALIGNMENT: usize = 1;

struct BumbAlloc {
    start: *mut u8,
    tracker: *mut u8,
    used_bytes: usize,
    layout: Layout,
}

impl BumbAlloc {
    fn build(length: usize) -> Self {
        let layout = Layout::from_size_align(length, ALIGNMENT).unwrap();

        let arena_size = layout.size();
        dbg!(&arena_size);
        println!();

        let ptr = unsafe { alloc_zeroed(layout) };

        Self {
            tracker: ptr,
            start: ptr,
            used_bytes: 0,
            layout,
        }
    }

    fn push(&mut self, layout: &Layout) -> *mut u8 {
        let prev_tracker: *mut u8;
        let requested_bytes = layout.size();
        dbg!(&requested_bytes);

        if self.layout.size() >= self.used_bytes + layout.align() {
            prev_tracker = self.tracker;

            let padding = self.tracker.align_offset(layout.align());

            // updating arena tracker pointer
            let offset = padding + requested_bytes;
            dbg!(&prev_tracker);
            unsafe {
                self.tracker = self.tracker.add(offset);
            }
            dbg!(&offset);
            dbg!(&self.tracker);

            // updating arena used bytes
            let new_bytes = padding + requested_bytes;
            self.used_bytes += new_bytes;
            dbg!(&self.used_bytes);
        } else {
            eprintln!("[ERROR] requested allocation is more then arena's remaining space\n",);
            prev_tracker = ptr::null_mut()
        };

        let remaining_space = self.layout.size().saturating_sub(self.used_bytes);
        dbg!(&remaining_space);
        println!();

        prev_tracker
    }

    fn uninitialize(&mut self) {
        self.used_bytes = 0;
        self.layout = Layout::new::<()>();
        self.start = ptr::null_mut();
        self.tracker = ptr::null_mut();
    }

    pub fn clear(&mut self) {
        unsafe { dealloc(self.start, self.layout) };
        self.uninitialize();
    }
}

impl Drop for BumbAlloc {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod test {
    const LENGTH: usize = 1000;

    use super::*;
    #[test]
    fn arena_size() {
        let arena = BumbAlloc::build(LENGTH);

        assert_eq!(arena.layout.size(), LENGTH);
    }

    #[test]
    fn not_enough_space_to_allocate() {
        let mut arena = BumbAlloc::build(0);
        let layout = Layout::new::<i32>();
        let ptr = arena.push(&layout);
        // not enough space, arena.used_bytes will not change
        assert_eq!(arena.used_bytes, 0);
        assert!(ptr.is_null());
    }

    #[test]
    fn bumbing_arena() {
        let mut arena = BumbAlloc::build(LENGTH);

        let layout = Layout::new::<i32>();
        let _ = arena.push(&layout);

        arena.clear();

        assert_eq!(arena.layout.size(), 0);
        // can't have alignment of 0 bytes in rust
        assert_eq!(arena.layout.align(), 1);
        assert_eq!(arena.used_bytes, 0);
        assert_eq!(arena.tracker, ptr::null_mut());
        assert_eq!(arena.start, ptr::null_mut());
    }

    #[test]
    fn arena_push_primitives() {
        let mut arena = BumbAlloc::build(LENGTH);

        let layout = Layout::new::<[i16; 2]>();
        let ptr_1 = arena.push(&layout).cast::<[i16; 2]>();
        assert_eq!(arena.used_bytes, 4);

        let layout = Layout::new::<[i64; 3]>();
        let ptr_2 = arena.push(&layout).cast::<[i64; 3]>();
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

        arena.clear();
    }

    #[test]
    fn arena_push_struct() {
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
        let mut arena = BumbAlloc::build(LENGTH);

        let ptr_1 = arena.push(&foo_layout).cast::<Foo>();
        assert_eq!(arena.used_bytes, foo_layout.size());

        let ptr_2 = arena.push(&boo_layout).cast::<Boo>();
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
        arena.clear();
    }
}
