#![allow(dead_code)]
mod log;
#[cfg(test)]
mod tests;

use log::*;
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr,
};

const ALIGNMENT: usize = 1;

struct Arena {
    start: *mut u8,
    tracker: *mut u8,
    used_bytes: usize,
    layout: Layout,
}

impl Arena {
    fn build(size: usize) -> Self {
        let layout = Layout::from_size_align(size, ALIGNMENT)
            .expect("requested length pass the boundry and wrapped to negative value");

        // size could be less or more depend if size is multiple of alignment or not
        let arena_size = layout.size();
        log_arena_size(arena_size);

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
    prev_allocation_size: usize,
}

impl StackAlloc {
    fn build(length: usize) -> Self {
        Self {
            arena: Arena::build(length),
            prev_allocation_size: 0,
            prev_tracker: ptr::null_mut(),
        }
    }

    fn push(&mut self, layout: &Layout) -> *mut u8 {
        let requested_bytes = layout.size();
        let padding = self.arena.tracker.align_offset(layout.align());
        let bytes_to_push = padding + requested_bytes;

        log_push(requested_bytes, padding);

        if self.arena.layout.size() >= self.arena.used_bytes + bytes_to_push {
            self.prev_tracker = self.arena.tracker;

            // update tracker
            let offset = padding + requested_bytes;
            unsafe {
                self.arena.tracker = self.arena.tracker.add(offset);
            }
            log_tracker(self.prev_tracker, self.arena.tracker);

            // update used bytes
            self.arena.used_bytes += bytes_to_push;
            log_used_bytes(self.arena.used_bytes - bytes_to_push, self.arena.used_bytes);

            // used later on pop method
            self.prev_allocation_size = bytes_to_push;
        } else {
            eprintln!("[ERROR] requested allocation is more then arena's remaining space",);
            // used to pop it
            self.prev_tracker = ptr::null_mut();
        };

        let remaining = self
            .arena
            .layout
            .size()
            .saturating_sub(self.arena.used_bytes);
        log_remaining_space(remaining);

        self.prev_tracker
    }

    fn pop(&mut self) {
        // update used_bytes
        log_pop(
            self.prev_tracker,
            self.arena.tracker,
            self.arena.used_bytes,
            self.prev_allocation_size,
        );

        self.arena.used_bytes = self
            .arena
            .used_bytes
            .saturating_sub(self.prev_allocation_size);

        // update tracker
        self.arena.tracker = self.prev_tracker;
    }

    fn uninitialize(&mut self) {
        self.arena.uninitialize();

        self.prev_tracker = ptr::null_mut();
        self.prev_allocation_size = 0;
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
        let requested_bytes = layout.size();
        let padding = self.arena.tracker.align_offset(layout.align());
        let bytes_to_push = padding + requested_bytes;

        log_push(requested_bytes, padding);

        // no need to make prev tracker as a field, as i only need it to
        //give it to the caller(unlick the stack allocator)
        let prev_tracker: *mut u8;

        if self.arena.layout.size() >= self.arena.used_bytes + bytes_to_push {
            prev_tracker = self.arena.tracker;

            // update tracker
            let offset = padding + requested_bytes;
            unsafe {
                self.arena.tracker = self.arena.tracker.add(offset);
            }
            log_tracker(prev_tracker, self.arena.tracker);

            // update used bytes
            self.arena.used_bytes += bytes_to_push;
            log_used_bytes(self.arena.used_bytes - bytes_to_push, self.arena.used_bytes);
        } else {
            eprintln!("[ERROR] requested allocation is more then arena's remaining space",);
            prev_tracker = ptr::null_mut()
        };

        let remaining = self
            .arena
            .layout
            .size()
            .saturating_sub(self.arena.used_bytes);
        log_remaining_space(remaining);

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
