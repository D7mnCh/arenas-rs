use super::arenas::{arena::Arena, pool_allocator::Block};
use std::alloc::Layout;

pub fn log_arena_build(layout: &Layout, start_ptr: *const u8) {
    println!("[ARENA BUILD]");

    let arena_size = layout.size();
    let arena_align = layout.align();
    dbg!(arena_size, arena_align);

    println!("arena start = {:p}", start_ptr);
    println!();
}

pub fn log_pool_remove(tracker: *const u8) {
    println!("[POOL POP]");
    println!("freeing a block from this tracker : {tracker:p}");
}

pub fn log_pool_alloc_build(num_blocks: usize, blocks: &[Block]) {
    println!("[POOL BUILD]");
    println!("num blocks = {num_blocks}");
    println!("block size = {size}", size = blocks[0].layout.size());
    println!();

    //_log_display_blocks_trackers(blocks);
}

pub fn _log_display_blocks_tracker(blocks: &[Block]) {
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
