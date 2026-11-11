use super::arena::Arena;
use crate::{
    arenas::arena::ArenaError,
    log::{
        log_adding_to_arena_to_match_num_blocks, log_free_blocks, log_pool_alloc_build,
        log_pool_remove,
    },
};
use std::{alloc::Layout, ptr::NonNull};

pub enum PoolAllocError {
    OutOfBoundRequest,
    PoolAllocFull,
    InvalidTracker,
    BlockAlreadyFree,
}

impl PoolAllocError {
    fn report(&self) {
        match self {
            PoolAllocError::OutOfBoundRequest => {
                println!("[WARNING] instance's size is bigger then block's size")
            }
            PoolAllocError::PoolAllocFull => println!("[WARNING] all blocks are reserved!"),
            PoolAllocError::InvalidTracker => {
                println!("[WARNING] tracker is not one of pool allocator's trackers\n")
            }

            PoolAllocError::BlockAlreadyFree => {
                println!("[WARNING] block is unused (free) from the giving tracker\n")
            }
        }
    }
}

#[derive(Debug)]
pub struct PoolAlloc {
    pub arena: Arena,
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub layout: Layout,
    pub tracker: NonNull<u8>,
    pub is_used: bool,
}

impl Block {
    pub fn build(layout: Layout, tracker: NonNull<u8>) -> Self {
        Self {
            layout,
            tracker,
            is_used: false,
        }
    }
}

impl PoolAlloc {
    // add to arena_size if arena_size % block_size != 0, to get zero
    //trailing padding, and arena_size be multiple of block_size
    fn add_to_arena_size(arena_size: usize, block_size: usize) -> usize {
        let modulor = arena_size % block_size;

        if modulor == 0 {
            return 0;
        }

        let added_bytes = block_size - modulor;
        log_adding_to_arena_to_match_num_blocks(added_bytes);
        return added_bytes;
    }

    fn construct_blocks(arena: &Arena, block_size: usize) -> Result<Vec<Block>, ArenaError> {
        let mut blocks: Vec<Block> = Vec::new();
        let first_block_tracker = arena.start.ok_or(ArenaError::ArenaBuildFailed)?;
        let num_blocks: usize = arena.layout.size() / block_size;

        for indx in 0..num_blocks {
            let block_align = block_size;
            let layout = Layout::from_size_align(block_size, block_align).expect(
                "requested length passed isize::MAX boundary \
                 and wrapped to negative value",
            );
            let block_tracker: NonNull<u8>;
            let offset = indx * block_size;
            unsafe {
                block_tracker = first_block_tracker.add(offset);
            }

            let block = Block::build(layout, block_tracker);
            blocks.push(block);
        }

        log_pool_alloc_build(num_blocks, &blocks);

        return Ok(blocks);
    }

    // to build, need to specify blocks's layout and there trackers
    pub fn build(mut arena_size: usize, block_size: usize) -> Result<Self, ArenaError> {
        arena_size += PoolAlloc::add_to_arena_size(arena_size, block_size);

        let arena = Arena::build(arena_size)?;
        let blocks = PoolAlloc::construct_blocks(&arena, block_size)?;

        Ok(Self { arena, blocks })
    }

    pub fn push(&mut self, layout: &Layout) -> Result<NonNull<u8>, PoolAllocError> {
        println!("[POOL PUSH]");

        // check for an available block
        let Some((indx, block)) = self
            .blocks
            .iter_mut()
            .enumerate()
            .find(|(_, block)| !block.is_used)
        else {
            log_free_blocks(&self.blocks);
            return Err(PoolAllocError::PoolAllocFull);
        };
        println!("[INFO] found tracker/free block");

        // check if requested bytes fits inside block's size
        if layout.size() > block.layout.size() {
            log_free_blocks(&self.blocks);
            return Err(PoolAllocError::OutOfBoundRequest);
        }

        block.is_used = true;
        log_free_blocks(&self.blocks);

        return Ok(self.blocks[indx].tracker);
    }

    pub fn remove(&mut self, tracker: NonNull<u8>) -> Result<(), PoolAllocError> {
        let Some(index) = self.found_block(tracker) else {
            return Err(PoolAllocError::InvalidTracker);
        };

        let block = &mut self.blocks[index];
        if !block.is_used {
            return Err(PoolAllocError::BlockAlreadyFree);
        }

        block.is_used = false;
        log_pool_remove(tracker);

        Ok(())
    }

    pub fn clear(&mut self) {
        self.arena.clear();
    }

    // using "position" instead of "find" method, if i wanna return a block i need "&mut self"
    //but i am just searching for a valid block so just "&self"
    fn found_block(&self, tracker: NonNull<u8>) -> Option<usize> {
        self.blocks
            .iter()
            .position(|block| block.tracker == tracker)
    }
}
