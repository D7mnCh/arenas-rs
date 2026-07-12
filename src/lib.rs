// TODO inroduce modules
// TODO inroduce associated functions to increase readability, instead of
// commenting

#![allow(dead_code)]
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
    pub fn build(size: usize) -> Self {
        // there's no need for alignment to be more then 1, only if the
        // programmer want more then one arena instance
        const ALIGNMENT: usize = 1;

        let layout = Layout::from_size_align(size, ALIGNMENT).expect(
            "requested length passed isize::MAX boundry \
            and wrapped to negative value",
        );

        let start_ptr = unsafe { alloc_zeroed(layout) };
        log_arena_build(&layout, start_ptr);

        Self {
            tracker: start_ptr,
            start: start_ptr,
            used_bytes: 0,
            layout,
        }
    }

    fn bytes_to_push(&self, layout: &Layout) -> usize {
        let requested_bytes = layout.size();
        let padding = self.tracker.align_offset(layout.align());
        let bytes = padding + requested_bytes;

        bytes
    }

    pub fn clear(&mut self) {
        unsafe { dealloc(self.start, self.layout) };
    }
}

#[derive(Debug)]
struct PoolAlloc {
    arena: Arena,
    blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
struct Block {
    layout: Layout,
    tracker: *mut u8,
    is_used: bool,
}

impl Block {
    pub fn build(layout: Layout, tracker: *mut u8) -> Self {
        Self {
            layout,
            tracker,
            is_used: false,
        }
    }
}

impl PoolAlloc {
    // to build need to specify blocks's layout and there trackers
    pub fn build(mut arena_size: usize, block_size: usize) -> Self {
        // add to arena_size if arena_size % block_size != 0, to get zero
        //trailling padding
        let modulor = arena_size % block_size;
        arena_size += if modulor != 0 {
            let add = block_size - modulor;
            println!(
                "[WARNING] adding {add} bytes to arena size \
                to make it a multiple of the number of blocks"
            );
            add
        } else {
            0
        };

        // get "start pointer", and to construct the allocator ofc
        let arena = Arena::build(arena_size);

        // constructing the blocks
        let mut blocks: Vec<Block> = Vec::new();
        let first_block_tracker = arena.start.clone();
        let num_blocks: usize = arena_size / block_size;
        for indx in 0..num_blocks {
            let block_align = block_size;
            let layout = Layout::from_size_align(block_size, block_align).expect(
                "requested length passed isize::MAX boundry \
                    and wrapped to negative value",
            );
            let block_tracker: *mut u8;
            let offset = indx * block_size;
            unsafe {
                block_tracker = first_block_tracker.add(offset);
            }
            let block = Block::build(layout, block_tracker);
            blocks.push(block);
        }

        log_pool_alloc_build(num_blocks, &blocks);

        Self { arena, blocks }
    }

    pub fn push(&mut self, layout: &Layout) -> *mut u8 {
        println!("[POOL PUSH]");

        if layout.size() <= self.blocks[0].layout.size() {
            for (indx, block) in self.blocks.iter_mut().enumerate() {
                // check if any block is available
                if !block.is_used {
                    println!("[INFO] found tracker/free block");
                    block.is_used = true;
                    log_free_blocks(&self.blocks);
                    return self.blocks[indx].tracker;
                }
            }
            eprintln!("[WARNING] all blocks are reserved!");
            log_free_blocks(&self.blocks);
            ptr::null_mut()
        } else {
            eprintln!("[WARNING] instance's size is bigger then block's size");
            log_free_blocks(&self.blocks);
            ptr::null_mut()
        }
    }

    pub fn remove(&mut self, tracker: *mut u8) {
        let check_block_valid = self.blocks.iter_mut().find(|x| x.tracker == tracker);

        if let Some(block) = check_block_valid {
            log_pool_pop(tracker, ptr::null_mut());
            if block.is_used {
                block.is_used = false;
            } else {
                eprintln!(
                    "[WARNINIG] block is unused (free) \
                    from the givin tracker\n"
                );
            }
        } else {
            eprintln!("[WARNING] tracker is not one of pool-allocator's trackers\n");
        }
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }
}

#[derive(Debug)]
struct StackAlloc {
    arena: Arena,
    prev_allocation_sizes: Vec<usize>,
    prev_trackers: Vec<*mut u8>,
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

        // update used_bytes by reducing it
        self.arena.used_bytes = self.arena.used_bytes.saturating_sub(prev_allocation_size);
        self.prev_allocation_sizes.pop();

        // update tracker by offset it backward
        self.arena.tracker = prev_tracker;
        self.prev_trackers.pop();
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }
}

struct BumpAlloc {
    arena: Arena,
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

mod log {
    use super::{Arena, Block};
    use std::alloc::Layout;

    pub fn log_arena_build(layout: &Layout, start_ptr: *const u8) {
        println!("[ARENA BUILD]");

        let arena_size = layout.size();
        let arena_align = layout.align();
        dbg!(arena_size, arena_align);

        println!("arena start = {:p}", start_ptr);
        println!();
    }

    pub fn log_pool_pop(old: *const u8, new: *const u8) {
        println!("[POOL POP]");
        log_update_tracker(old, new);
        println!("[INFO] tracker : {old:p} -> {new:p}");
        println!("[INFO] reset block from that tracker \"{old:p}\"");
    }

    pub fn log_pool_alloc_build(num_blocks: usize, blocks: &[Block]) {
        println!("[POOL BUILD]");
        println!("num blocks = {num_blocks}");
        println!("block size = {size}", size = blocks[0].layout.size());
        println!();

        //log_show_blocks_trackers(blocks);
    }

    pub fn _log_show_blocks_tracker(blocks: &[Block]) {
        println!("trackers :");
        for (mut indx, block) in blocks.iter().enumerate() {
            indx += 1;
            println!("  {tracker:p} -> {indx} block", tracker = block.tracker);
        }
        println!();
    }

    pub fn log_free_blocks(blocks: &[Block]) {
        let mut free_blocks = 0;

        for block in blocks {
            if !block.is_used {
                free_blocks += 1;
            }
        }

        println!("free blocks = {free_blocks}",);
        println!();
    }

    pub fn log_push(alloc: &str, arena: &Arena, layout: &Layout) {
        let requested_bytes = layout.size();
        let padding = arena.tracker.align_offset(layout.align());

        println!("[{alloc} PUSH]");

        dbg!(requested_bytes, padding);

        let total = requested_bytes + padding;
        println!("total = {} b", total);
    }

    pub fn log_stack_pop(
        old_tracker: *const u8,
        new_tracker: *const u8,
        old_used_bytes: usize,
        current_used_bytes: usize,
    ) {
        let new_used_bytes = current_used_bytes.saturating_sub(old_used_bytes);

        println!("[STACK POP]");
        println!("tracker    = {new_tracker:p} -> {old_tracker:p}");
        println!("used_bytes = {current_used_bytes} -> {new_used_bytes}");
        println!();
    }

    pub fn log_update_tracker(old: *const u8, new: *const u8) {
        println!("tracker = {old:p} -> {new:p}",);
    }

    pub fn log_update_used_bytes(bytes_to_push: usize, new: usize) {
        let old = new - bytes_to_push;
        println!("used_bytes = {old} -> {new}");
    }

    pub fn log_remaining_space(arena: &Arena) {
        let remaining = arena.layout.size().saturating_sub(arena.used_bytes);
        println!("remaining = {remaining}");
        println!();
    }
}
