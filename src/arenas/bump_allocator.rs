use super::arena::Arena;
use crate::{
    arenas::arena::ArenaError,
    log::{log_remaining_space, log_update_tracker, log_update_used_bytes},
};
use std::{alloc::Layout, ptr::NonNull};

pub struct BumpAlloc {
    pub arena: Arena,
}

impl BumpAlloc {
    pub fn build(length: usize) -> Result<Self, ArenaError> {
        Ok(Self {
            arena: Arena::build(length)?,
        })
    }

    pub fn push(&mut self, layout: &Layout) -> Result<NonNull<u8>, ArenaError> {
        let tracker = self.arena.tracker.ok_or(ArenaError::TryUseOfUnbuiltArena)?;

        let bytes_to_push = self.arena.bytes_to_push(&layout)?;
        let prev_tracker = tracker;

        self.arena.push(layout, bytes_to_push)?;

        log_update_tracker(prev_tracker, tracker);
        log_update_used_bytes(bytes_to_push, self.arena.used_bytes);
        log_remaining_space(&self.arena);

        Ok(prev_tracker)
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }
}
