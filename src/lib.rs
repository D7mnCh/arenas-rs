#![allow(dead_code)]
mod log;
#[cfg(test)]
mod tests;

use log::*;
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ptr,
};

#[derive(Debug)]
struct Arena {
    start: *mut u8,
    tracker: *mut u8,
    used_bytes: usize,
    layout: Layout,
}

impl Arena {
    fn build(size: usize) -> Self {
        // there's no need for alignment to be more then 1, only if the
        //caller want more then one arena
        const ALIGNMENT: usize = 1;

        let layout = Layout::from_size_align(size, ALIGNMENT)
            .expect("requested length pass isize::MAX boundry and wrapped to negative value");

        let start_ptr = unsafe { alloc_zeroed(layout) };
        log_arena_info(&layout, start_ptr);

        Self {
            tracker: start_ptr,
            start: start_ptr,
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

// NOTE poll allocator only have trailling padding
#[derive(Debug)]
struct PollAlloc {
    arena: Arena,
    // each block share the same layout
    blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
struct Block {
    layout: Layout,
    tracker: *mut u8,
    is_used: bool,
}

impl Block {
    fn build(layout: Layout, tracker: *mut u8) -> Self {
        Self {
            layout,
            tracker,
            is_used: false,
        }
    }
}

impl PollAlloc {
    // while building, need to specify blocks's layout and there
    //trackers
    fn build(mut arena_size: usize, block_size: usize) -> Self {
        // add to arena_size if arena_size % block_size != 0
        let modulor = arena_size % block_size;
        arena_size += block_size - modulor;
        let num_blocks: usize = arena_size / block_size;

        // need "start pointer" from arena to construct the blocks trackers
        let arena = Arena::build(arena_size);
        // constructing the blocks
        let mut blocks: Vec<Block> = Vec::new();
        let first_block_tracker = arena.start.clone();
        let mut offset = 0;

        for _block in 0..num_blocks {
            let block_align = block_size;
            let layout = Layout::from_size_align(block_size, block_align)
                .expect("requested length pass isize::MAX boundry and wrapped to negative value");
            let block_tracker: *mut u8;
            unsafe {
                block_tracker = first_block_tracker.add(offset);
            }
            let block = Block::build(layout, block_tracker);
            blocks.push(block);
            offset += block_size;
        }

        Self { arena, blocks }
    }

    fn push(&mut self, layout: &Layout) -> *mut u8 {
        if layout.size() < self.blocks[0].layout.size() {
            for (indx, block) in self.blocks.iter_mut().enumerate() {
                // check if any block is available
                if !block.is_used {
                    block.is_used = true;
                    return self.blocks[indx].tracker;
                }
            }
            eprintln!("[WARNING] all blocks are reserved!");
            return ptr::null_mut();
        } else {
            eprintln!("[WARNING] instance size is bigger then block size");
            return ptr::null_mut();
        }
    }

    // NOTE i think the caller give a tracker, then match that tracker
    // to pop it out from the arena
    fn pop(_tracker: *mut u8) {}

    fn uninitialize() {}

    fn clear() {}
}

struct StackAlloc {
    arena: Arena,
    prev_allocation_sizes: Vec<usize>,
    prev_trackers: Vec<*mut u8>,
}

impl StackAlloc {
    fn build(size: usize) -> Self {
        Self {
            arena: Arena::build(size),
            prev_allocation_sizes: Vec::new(),
            prev_trackers: Vec::new(),
        }
    }

    fn push(&mut self, layout: &Layout) -> *mut u8 {
        let requested_bytes = layout.size();
        let padding = self.arena.tracker.align_offset(layout.align());
        let bytes_to_push = padding + requested_bytes;

        log_push(requested_bytes, padding);

        if self.arena.layout.size() >= self.arena.used_bytes + bytes_to_push {
            // prev_trackers used on pop method
            self.prev_trackers.push(self.arena.tracker);
            // safe to unwrap, cuz i am pushing to prev_trackers before
            //i use it, so always will be a value inside the colliction
            let prev_tracker = self.prev_trackers.last().unwrap().to_owned();

            // update tracker
            let offset = padding + requested_bytes;
            unsafe {
                self.arena.tracker = self.arena.tracker.add(offset);
            }
            log_tracker(prev_tracker, self.arena.tracker);

            // update used bytes
            self.arena.used_bytes += bytes_to_push;
            log_used_bytes(self.arena.used_bytes - bytes_to_push, self.arena.used_bytes);

            // prev_allocation_sizes used on pop method
            self.prev_allocation_sizes.push(bytes_to_push);

            let remaining = self
                .arena
                .layout
                .size()
                .saturating_sub(self.arena.used_bytes);
            log_remaining_space(remaining);

            return prev_tracker;
        } else {
            log_warning_arena_is_full();
            return ptr::null_mut();
        }
    }

    fn pop(&mut self) {
        if self.arena.used_bytes != 0 {
            // safe to call unwrap cuz if used_bytes != 0, there's an element
            // inside the collictions
            let prev_tracker = self.prev_trackers.last().unwrap().to_owned();
            let prev_allocation_size = self.prev_allocation_sizes.last().unwrap().to_owned();

            // if get not "enough space" warning when tried to push or there's no used_bytes
            // then don't pop(caller can call pop before even push method)
            log_pop(
                prev_tracker,
                self.arena.tracker,
                prev_allocation_size,
                self.arena.used_bytes,
            );

            // update used_bytes
            self.arena.used_bytes = self.arena.used_bytes.saturating_sub(prev_allocation_size);
            self.prev_allocation_sizes.pop();

            // update tracker
            self.arena.tracker = prev_tracker;
            self.prev_trackers.pop();
        } else {
            log_warning_arena_is_empty();
        }
    }

    fn uninitialize(&mut self) {
        self.arena.uninitialize();

        self.prev_trackers = Vec::new();
        self.prev_allocation_sizes = Vec::new();
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
            log_warning_arena_is_full();
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
