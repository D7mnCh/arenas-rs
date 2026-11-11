use crate::log::{log_arena_build, log_remaining_space};
use std::{
    alloc::{Layout, LayoutError, alloc_zeroed, dealloc},
    ptr::NonNull,
};

pub enum ArenaError {
    UnexpectedLayoutArenaBuild(LayoutError),
    AllocationFailed,
    ArenaBuildFailed,
    ArenaFull,
    TryUseOfUnbuiltArena,
    ArenaEmpty,
}

// NOTE didn't really know why should the caller report and not me ?
impl ArenaError {
    fn report(&self) {
        match self {
            ArenaError::AllocationFailed => println!("[WARNING] arena allocation failed"),
            ArenaError::ArenaFull => println!(
                "[WARNING] requested allocation \
                is more then arena's remaining space\n",
            ),
            ArenaError::ArenaBuildFailed => println!("[WARNING] arena build failed"),
            ArenaError::TryUseOfUnbuiltArena => println!("[WARNING] arena is unallocated to use"),
            ArenaError::UnexpectedLayoutArenaBuild(err) => {
                println!("[WARNING] failed to build arena cuzed by unexpected Layout: {err} ")
            }
            ArenaError::ArenaEmpty => println!("[WARNING] arena empty"),
        }
    }
}

#[derive(Debug)]
pub struct Arena {
    // i chooce this pointer type cuz it represent the pointer
    // (not chosen for performance tho :p).
    // Option used for resetting, NonNull have no default value, so should be None
    pub start: Option<NonNull<u8>>,
    pub tracker: Option<NonNull<u8>>,
    pub used_bytes: usize,
    pub layout: Layout,
}

impl Arena {
    pub fn build(size: usize) -> Result<Self, ArenaError> {
        // there's no need for alignment to be more then 1, only if the
        // programmer want more then one arena instance.
        // NOTE i think the programmer will need more then one arena -.-
        const ALIGNMENT: usize = 1;

        let layout = Layout::from_size_align(size, ALIGNMENT)
            .map_err(|err| ArenaError::UnexpectedLayoutArenaBuild(err))?;
        let start_ptr =
            NonNull::new(unsafe { alloc_zeroed(layout) }).ok_or(ArenaError::AllocationFailed)?;

        log_arena_build(&layout, start_ptr);

        Ok(Self {
            tracker: Some(start_ptr),
            start: Some(start_ptr),
            used_bytes: 0,
            layout,
        })
    }

    pub fn bytes_to_push(&self, layout: &Layout, padding: usize) -> usize {
        let requested_bytes = layout.size();
        let bytes = padding + requested_bytes;

        bytes
    }

    pub fn push(&mut self, bytes_to_push: usize) -> Result<(), ArenaError> {
        let tracker = self.tracker.ok_or(ArenaError::TryUseOfUnbuiltArena)?;

        if self.layout.size() < self.used_bytes + bytes_to_push {
            log_remaining_space(&self);
            return Err(ArenaError::ArenaFull);
        }

        let offset = bytes_to_push;
        self.tracker = Some(unsafe { tracker.add(offset) });
        self.used_bytes += bytes_to_push;

        Ok(())
    }

    pub fn clear(&mut self) {
        if let Some(start_ptr) = self.start {
            unsafe { dealloc(start_ptr.as_ptr(), self.layout) };
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.tracker = None;
        self.start = None;
        self.used_bytes = 0;
        self.layout = Layout::new::<()>();
    }
}
