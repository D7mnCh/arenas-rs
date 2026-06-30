use std::alloc::Layout;

pub fn log_arena_info(layout: &Layout, start_ptr: *const u8) {
    println!("[BUILD]");
    println!("arena size      = {} B", layout.size());
    println!("arena alignment = {}", layout.align());
    println!("arena start     = {:p}", start_ptr);
    println!();
}

pub fn log_push(request: usize, padding: usize) {
    let total = request + padding;

    println!("[PUSH]");
    println!("request    = {} b", request);
    println!("padding    = {} b", padding);
    println!("total      = {} b", total);
}

pub fn log_warning_arena_is_full() {
    eprintln!("[WARNING] requested allocation is more then arena's remaining space",);
}
pub fn log_warning_arena_is_empty() {
    eprintln!("[WARNING] can't pop, arena is empty!");
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

pub fn log_tracker(old: *const u8, new: *const u8) {
    println!("tracker    = {old:p} -> {new:p}");
}

pub fn log_used_bytes(old: usize, new: usize) {
    println!("used_bytes = {old} -> {new}");
}

pub fn log_remaining_space(remaining: usize) {
    println!("remaining  = {remaining}");
    println!();
}
