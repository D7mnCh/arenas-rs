#![allow(dead_code)]
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr,
};
#[cfg(test)]
mod tests;

const ALIGNMENT: usize = 1;

macro_rules! trace_alloc {
    ($func_name:literal, $($x:expr),*) => {
        println!("{}", $func_name);
        $(
            dbg!($x);
         )*
           println!();
    };
}
struct Arena {
    start: *mut u8,
    tracker: *mut u8,
    used_bytes: usize,
    layout: Layout,
}

impl Arena {
    fn build(length: usize) -> Self {
        let layout = Layout::from_size_align(length, ALIGNMENT)
            .expect("requested length pass the boundry and wrapped to negative value");

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

    fn uninitialize(&mut self) {
        self.used_bytes = 0;
        self.layout = Layout::new::<()>();
        self.start = ptr::null_mut();
        self.tracker = ptr::null_mut();
    }

    fn clear(&mut self) {
        unsafe { dealloc(self.start, self.layout) };
    }
}

struct StackAlloc {
    arena: Arena,
    prev_tracker: *mut u8,
    prev_data_size: usize,
}

impl StackAlloc {
    fn build(length: usize) -> Self {
        Self {
            arena: Arena::build(length),
            prev_data_size: 0,
            prev_tracker: ptr::null_mut(),
        }
    }

    fn push(&mut self, layout: &Layout) -> *mut u8 {
        println!("[INFO] request to push data into the arena");

        let requested_bytes = layout.size();

        let padding = self.arena.tracker.align_offset(layout.align());

        let bytes_to_push = padding + requested_bytes;

        dbg!(&requested_bytes);
        dbg!(&padding);
        dbg!(&bytes_to_push);

        if self.arena.layout.size() >= self.arena.used_bytes + bytes_to_push {
            self.prev_tracker = self.arena.tracker;

            // update tracker
            let offset = padding + requested_bytes;
            dbg!(&self.prev_tracker);
            dbg!(&offset);
            unsafe {
                self.arena.tracker = self.arena.tracker.add(offset);
            }
            dbg!(&self.arena.tracker);

            // update used bytes
            dbg!(&self.arena.used_bytes);
            self.arena.used_bytes += bytes_to_push;
            dbg!(&self.arena.used_bytes);

            // used to pop it
            self.prev_data_size = bytes_to_push;
        } else {
            eprintln!("[ERROR] requested allocation is more then arena's remaining space\n",);
            // used to pop it
            self.prev_tracker = ptr::null_mut();
        };

        let remaining_space = self
            .arena
            .layout
            .size()
            .saturating_sub(self.arena.used_bytes);

        dbg!(&remaining_space);
        println!();

        self.prev_tracker
    }

    fn pop(&mut self) {
        println!("[INFO] request to pop arena");
        // update used_bytes
        dbg!(&self.arena.used_bytes);
        self.arena.used_bytes = self.arena.used_bytes.saturating_sub(self.prev_data_size);
        dbg!(&self.arena.used_bytes);

        // update tracker
        dbg!(&self.arena.tracker);
        self.arena.tracker = self.prev_tracker;
        dbg!(&self.arena.tracker);
        println!();
    }

    fn uninitialize(&mut self) {
        self.arena.uninitialize();

        self.prev_tracker = ptr::null_mut();
        self.prev_data_size = 0;
    }

    pub fn clear(&mut self) {
        self.arena.clear();
        self.uninitialize();
    }
}

impl Drop for StackAlloc {
    fn drop(&mut self) {
        self.clear();
    }
}

struct BumbAlloc {
    arena: Arena,
}

impl BumbAlloc {
    fn build(length: usize) -> Self {
        Self {
            arena: Arena::build(length),
        }
    }

    fn push(&mut self, layout: &Layout) -> *mut u8 {
        println!("[INFO] request to push data into the arena");

        let requested_bytes = layout.size();
        dbg!(&requested_bytes);

        let padding = self.arena.tracker.align_offset(layout.align());
        dbg!(&padding);
        let bytes_to_push = padding + requested_bytes;
        dbg!(&bytes_to_push);

        // no need to make prev tracker as a field, as i only need it to
        //give it to the caller(unlick the stack allocator)
        let prev_tracker: *mut u8;

        if self.arena.layout.size() >= self.arena.used_bytes + bytes_to_push {
            prev_tracker = self.arena.tracker;

            // update tracker
            let offset = padding + requested_bytes;
            dbg!(&prev_tracker);
            dbg!(&offset);
            unsafe {
                self.arena.tracker = self.arena.tracker.add(offset);
            }
            dbg!(&self.arena.tracker);

            // update used bytes
            dbg!(&self.arena.used_bytes);
            self.arena.used_bytes += bytes_to_push;
            dbg!(&self.arena.used_bytes);
        } else {
            eprintln!("[ERROR] requested allocation is more then arena's remaining space\n",);
            prev_tracker = ptr::null_mut()
        };

        let remaining_space = self
            .arena
            .layout
            .size()
            .saturating_sub(self.arena.used_bytes);
        dbg!(&remaining_space);
        println!();

        prev_tracker
    }

    fn uninitialize(&mut self) {
        self.arena.uninitialize();
    }

    pub fn clear(&mut self) {
        self.arena.clear();
        self.uninitialize();
    }
}

impl Drop for BumbAlloc {
    fn drop(&mut self) {
        self.clear();
    }
}
