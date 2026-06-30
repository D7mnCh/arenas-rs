pub fn log_push(request: usize, padding: usize) {
    let total = request + padding;

    println!("[PUSH]");
    println!("request    = {} b", request);
    println!("padding    = {} b", padding);
    println!("total      = {} b", total);
}

pub fn log_arena_size(size: usize) {
    println!("arena size = {size} B\n");
}

pub fn log_pop(
    old_tracker: *const u8,
    new_tracker: *const u8,
    old_used_bytes: usize,
    prev_allocation_size: usize,
) {
    let new_used_bytes = old_used_bytes.saturating_sub(prev_allocation_size);

    println!("[POP]");
    println!("tracker    = {old_tracker:p} -> {new_tracker:p}");
    println!("used_bytes = {old_used_bytes} -> {new_used_bytes}\n");
}

pub fn log_tracker(old: *mut u8, new: *mut u8) {
    println!("tracker    = {old:p} -> {new:p}");
}

pub fn log_used_bytes(old: usize, new: usize) {
    println!("used_bytes = {old} -> {new}");
}

pub fn log_remaining_space(remaining: usize) {
    println!("remaining  = {remaining}\n");
}
