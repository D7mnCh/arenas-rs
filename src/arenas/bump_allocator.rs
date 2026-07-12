use super::arena::Arena;
use crate::log::{log_push, log_remaining_space, log_update_tracker, log_update_used_bytes};
use std::{alloc::Layout, ptr};

pub struct BumpAlloc {
    pub arena: Arena,
}

impl BumpAlloc {
    pub fn build(length: usize) -> Self {
        Self {
            arena: Arena::build(length),
        }
    }

    // NOTE the only difference between stack and bump that they both push
    // methods is 2-LOC
    pub fn push(&mut self, layout: &Layout) -> *mut u8 {
        let bytes_to_push = self.arena.bytes_to_push(&layout);
        log_push("BUMB", &self.arena, &layout);

        if self.arena.layout.size() < self.arena.used_bytes + bytes_to_push {
            eprintln!(
                "[WARNING] requested allocation \
                is more then arena's remaining space\n",
            );
            let prev_tracker = ptr::null_mut();
            log_remaining_space(&self.arena);

            return prev_tracker;
        }

        let prev_tracker = self.arena.tracker;

        // update tracker
        let offset = bytes_to_push;
        unsafe {
            self.arena.tracker = self.arena.tracker.add(offset);
        }
        log_update_tracker(prev_tracker, self.arena.tracker);

        // update used bytes
        self.arena.used_bytes += bytes_to_push;
        log_update_used_bytes(bytes_to_push, self.arena.used_bytes);

        log_remaining_space(&self.arena);

        prev_tracker
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }
}
