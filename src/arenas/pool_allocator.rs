use super::arena::Arena;
use crate::log::{log_free_blocks, log_pool_alloc_build, log_pool_pop};
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
