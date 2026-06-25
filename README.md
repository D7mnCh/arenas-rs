# arenas-rs
Building arenas (region-based) memorey allocators in rust, implementing bumb, stack, free-list and pool allocator

> [!NOTE]
> most of the time when i say "allocate" i meant to allocate some block "inside" the region/arena

# Concepts
## general purpose allocator
it is the simpliest allocator, it just allocates or deallactes memory, no fancy methods or algorithms, in c or rust, it is the default allocator
## Bumb/leaner allocator
- it basically can allocate memory leanery only in one direction, that's means you can't like deallocate last allocation.
> [!NOTE]
> if you want an allocator that deallocate memory on the last allocation on the arena, consider reading the next allocator (the stack allocator)
- if you want deallocation, you'll need to deallocate all the region/arena all in once (that's where the name "bumb" came from!)
### implimentation,what you gonna need ?
- "pointer" to where this bumb allocator "start", in order to deallocate(bumbing) it later
- a "tracker pointer" that tracks the offset in terms of pointers so i can get the requested data by that pointer
- an "offset bytes size counter" that keep track of "how many bytes" that are allocated to check if you can allocate more size, by comparing it with "arena size" and by adding with the "requested-bytes"
### where to use


# What i learned

- rust std returend `u8` as a pointee type, cuz it's easier to deal with pointers arithmetics (only in pure bytes: u8 == 1 byte!)
- `memory alignment` referes to how data are arranged in memory
- `alignemnt` refers to num of how many bytes/blocks that a size/chunk should have so the last one will get full fill.If the size % alignment == 0, 
