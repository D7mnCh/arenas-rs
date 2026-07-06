use std::alloc::Layout;

// NOTE i'll use log functions when the body is long, if it just one line
//of code then write it inside the caller function
// TODO remove functions that have one line of code
// NOTE i should note what i learn beside project concepts

pub fn log_arena_info(layout: &Layout, start_ptr: *const u8) {
    println!("[BUILD]");

    let arena_size = layout.size();
    let arena_align = layout.align();
    dbg!(arena_size, arena_align);

    println!("arena start     = {:p}", start_ptr);
    println!();
}

pub fn log_push(request: usize, padding: usize) {
    println!("[PUSH]");

    dbg!(request, padding);

    let total = request + padding;
    println!("total_to_push = {} b", total);
}

pub fn log_pop(
    old_tracker: *const u8,
    new_tracker: *const u8,
    old_used_bytes: usize,
    current_used_bytes: usize,
) {
    let new_used_bytes = current_used_bytes.saturating_sub(old_used_bytes);

    println!("[POP]");
    println!("tracker    = {new_tracker:p} -> {old_tracker:p}");
    println!("used_bytes = {current_used_bytes} -> {new_used_bytes}");
    println!();
}

pub fn log_used_bytes(bytes_to_push: usize, new: usize) {
    let old = new - bytes_to_push;
    println!("used_bytes = {old} -> {new}");
}

pub fn log_remaining_space(remaining: usize) {
    println!("remaining  = {remaining}");
    println!();
}
