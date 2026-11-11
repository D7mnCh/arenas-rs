use super::arena::Arena;
use crate::{
    arenas::arena::ArenaError,
    log::{
        log_push, log_remaining_space, log_stack_pop, log_update_tracker, log_update_used_bytes,
    },
};
use std::{alloc::Layout, ptr::NonNull};

#[derive(Debug)]
pub struct StackAlloc {
    pub arena: Arena,
    pub prev_allocation_sizes: Vec<usize>,
    pub prev_trackers: Vec<NonNull<u8>>,
}

impl StackAlloc {
    pub fn build(size: usize) -> Result<Self, ArenaError> {
        Ok(Self {
            arena: Arena::build(size)?,
            prev_allocation_sizes: Vec::new(),
            prev_trackers: Vec::new(),
        })
    }

    pub fn push(&mut self, layout: &Layout) -> Result<NonNull<u8>, ArenaError> {
        let tracker = &self.arena.tracker.ok_or(ArenaError::TryUseOfUnbuiltArena)?;

        let padding = tracker.align_offset(layout.align());
        let bytes_to_push = self.arena.bytes_to_push(&layout, padding);

        log_push("STACK PUSH", *tracker, &layout);

        let prev_tracker = *tracker;
        self.arena.push(bytes_to_push)?;

        // prev_trackers used on pop() to offset arena's tracker to where he was last time
        self.prev_trackers.push(prev_tracker);

        log_update_tracker(prev_tracker, *tracker);
        log_update_used_bytes(self.arena.used_bytes - bytes_to_push, self.arena.used_bytes);
        log_remaining_space(&self.arena);

        // prev_allocation_sizes used on pop method to reduce used_bytes to where it was last time
        self.prev_allocation_sizes.push(bytes_to_push);

        Ok(prev_tracker)
    }

    pub fn pop(&mut self) -> Result<(), ArenaError> {
        let tracker = self.arena.tracker.ok_or(ArenaError::TryUseOfUnbuiltArena)?;

        if self.arena.used_bytes == 0 {
            return Err(ArenaError::ArenaEmpty);
        }

        // safe to call unwrap cuz if used_bytes != 0, there's alywas an
        //element inside the colliction
        let prev_tracker = *self.prev_trackers.last().expect("should used_bytes != 0");
        let prev_allocation_size = *self
            .prev_allocation_sizes
            .last()
            .expect("should used_bytes != 0");

        log_stack_pop(
            prev_tracker,
            tracker,
            prev_allocation_size,
            self.arena.used_bytes,
        );

        self.backward_tracker(prev_tracker);
        self.reduce_used_bytes(prev_allocation_size);

        Ok(())
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }

    fn backward_tracker(&mut self, prev_tracker: NonNull<u8>) {
        if let Some(ref mut tracker) = self.arena.tracker {
            *tracker = prev_tracker;
            self.prev_trackers.pop();
        }
    }

    fn reduce_used_bytes(&mut self, prev_allocation_size: usize) {
        self.arena.used_bytes = self.arena.used_bytes.saturating_sub(prev_allocation_size);
        self.prev_allocation_sizes.pop();
    }
}
