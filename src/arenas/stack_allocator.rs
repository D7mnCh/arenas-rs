use super::arena::Arena;
use crate::log::{
    log_push, log_remaining_space, log_stack_pop, log_update_tracker, log_update_used_bytes,
};
use std::{alloc::Layout, ptr};

#[derive(Debug)]
pub struct StackAlloc {
    pub arena: Arena,
    pub prev_allocation_sizes: Vec<usize>,
    pub prev_trackers: Vec<*mut u8>,
}

impl StackAlloc {
    pub fn build(size: usize) -> Self {
        Self {
            arena: Arena::build(size),
            prev_allocation_sizes: Vec::new(),
            prev_trackers: Vec::new(),
        }
    }

    pub fn push(&mut self, layout: &Layout) -> *mut u8 {
        let bytes_to_push = self.arena.bytes_to_push(&layout);
        log_push("BUMB", &self.arena, &layout);

        if self.arena.layout.size() < self.arena.used_bytes + bytes_to_push {
            eprintln!(
                "[WARNING] requested allocation \
                is more then arena's remaining space\n"
            );
            let prev_tracker = ptr::null_mut();
            log_remaining_space(&self.arena);

            return prev_tracker;
        }
        // prev_trackers used on pop method to offset arena tracker to where
        //he was last time
        self.prev_trackers.push(self.arena.tracker);

        let prev_tracker = self.arena.tracker;

        // update tracker
        let offset = bytes_to_push;
        unsafe {
            self.arena.tracker = self.arena.tracker.add(offset);
        }
        log_update_tracker(prev_tracker, self.arena.tracker);

        // update used bytes
        self.arena.used_bytes += bytes_to_push;
        log_update_used_bytes(self.arena.used_bytes - bytes_to_push, self.arena.used_bytes);

        // prev_allocation_sizes used on pop method to reduce used_bytes to
        //where it was last time
        self.prev_allocation_sizes.push(bytes_to_push);

        log_remaining_space(&self.arena);

        prev_tracker
    }

    pub fn pop(&mut self) {
        if self.arena.used_bytes == 0 {
            return eprintln!("[WARNING] can't pop, arena is empty!\n");
        }

        // safe to call unwrap cuz if used_bytes != 0, there's alywas an
        //element inside the colliction
        let prev_tracker = self.prev_trackers.last().unwrap().to_owned();
        let prev_allocation_size = self.prev_allocation_sizes.last().unwrap().to_owned();

        log_stack_pop(
            prev_tracker,
            self.arena.tracker,
            prev_allocation_size,
            self.arena.used_bytes,
        );

        self.backward_tracker(prev_tracker);
        self.reduce_used_bytes(prev_allocation_size);
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }

    fn backward_tracker(&mut self, prev_tracker: *mut u8) {
        self.arena.tracker = prev_tracker;
        self.prev_trackers.pop();
    }

    fn reduce_used_bytes(&mut self, prev_allocation_size: usize) {
        self.arena.used_bytes = self.arena.used_bytes.saturating_sub(prev_allocation_size);
        self.prev_allocation_sizes.pop();
    }
}
