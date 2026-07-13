use super::arena::Arena;
use crate::log::{log_free_blocks, log_pool_alloc_build, log_pool_remove};
use std::{alloc::Layout, ptr};

#[derive(Debug)]
pub struct PoolAlloc {
    pub arena: Arena,
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub layout: Layout,
    pub tracker: *mut u8,
    pub is_used: bool,
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
    // add to arena_size if arena_size % block_size != 0, to get zero
    //trailling padding, and arena_size be multiple of block_size
    fn add_to_arena_size(arena_size: usize, block_size: usize) -> usize {
        let modulor = arena_size % block_size;

        if modulor == 0 {
            return 0;
        }

        let add = block_size - modulor;
        println!(
            "[WARNING] adding {add} bytes to arena size to make it \
             a multiple of the number of blocks"
        );
        return add;
    }

    // NOTE i think i can use array instead to construct the vec
    fn construct_blocks(arena: &Arena, block_size: usize) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let first_block_tracker = arena.start.clone();
        let num_blocks: usize = arena.layout.size() / block_size;

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

        return blocks;
    }

    // to build, need to specify blocks's layout and there trackers
    pub fn build(mut arena_size: usize, block_size: usize) -> Self {
        arena_size += PoolAlloc::add_to_arena_size(arena_size, block_size);

        let arena = Arena::build(arena_size);
        let blocks = PoolAlloc::construct_blocks(&arena, block_size);

        Self { arena, blocks }
    }

    pub fn push(&mut self, layout: &Layout) -> *mut u8 {
        println!("[POOL PUSH]");

        if layout.size() > self.blocks[0].layout.size() {
            eprintln!("[WARNING] instance's size is bigger then block's size");
            log_free_blocks(&self.blocks);
            return ptr::null_mut();
        }

        // check for available block
        let Some((indx, block)) = self
            .blocks
            .iter_mut()
            .enumerate()
            .find(|(_indx, block)| !block.is_used)
        else {
            eprintln!("[WARNING] all blocks are reserved!");
            log_free_blocks(&self.blocks);
            return ptr::null_mut();
        };

        println!("[INFO] found tracker/free block");
        block.is_used = true;
        log_free_blocks(&self.blocks);
        return self.blocks[indx].tracker;
    }

    pub fn remove(&mut self, tracker: *const u8) {
        let Some(index) = self.found_block(tracker) else {
            eprintln!("[WARNING] tracker is not one of pool allocator's trackers\n");
            return;
        };

        let block = &mut self.blocks[index];
        if !block.is_used {
            eprintln!("[WARNINIG] block is unused (free) from the givin tracker\n");
        }

        block.is_used = false;
        log_pool_remove(tracker);
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }

    // using "position" instead of "find" method, if i wanna return a block i need "&mut self"
    //but i am just searching for a valid block so just "&self"
    fn found_block(&self, tracker: *const u8) -> Option<usize> {
        self.blocks
            .iter()
            .position(|block| block.tracker.cast_const() == tracker)
    }
}
