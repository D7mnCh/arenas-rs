# arenas-rs
Building arenas (region-based) memorey allocators in rust, implementing bumb, stack, free-list and pool allocator

> [!NOTE]
> most of the time when i say "allocate" i meant to allocate some block "inside" the region/arena

# Concepts
## general purpose allocator
it is the simpliest allocator, it just allocates or deallactes memory, no fancy methods or algorithms, in c or rust, it is the default allocator
## Bumb/leaner allocator (need re-write)
- it basically can allocate memory leanery only in one direction, that's means you can't like deallocate prev allocation.
> [!NOTE]
> if you want an allocator that deallocate memory on the last allocation on the arena, consider reading the next allocator (the stack allocator)
- if you want deallocation, you'll need to deallocate all the region/arena all in once (that's where the name "bumb" came from!)
### implimentation,what you gonna need ?
- "pointer" to where this bumb allocator "start", in order to deallocate(bumbing) it later
- a "tracker pointer" that tracks the offset in terms of pointers so i can get the requested data by that pointer
- an "offset bytes size counter" that keep track of "how many bytes" that are allocated to check if you can allocate more size, by comparing it with "arena size" and by adding with the "requested-bytes"
### where to use


# What i learned

- each block in memroy represent an address, we deal with addresses as bytes, and hexdecimal number
- rust std returend `u8` as a pointee type, cuz it's easier to deal with pointers arithmetics (only in pure bytes: u8 == 1 byte!)
- a `layout` is a description/request of a chunk of memory(no allocation yet), that request a size and an alignment
    - the required alignment is just an information, you'll impl the offset of the pointer manually to make an aligned data, the length is need for allocation
- `memory alignment` describes where piece of data should store in memory, a data is properly aligned if its "starting address" is a "multiple" of its "required alignemnt", when this condition satisfied, we say "the data is "aligned" or "properley aligned""
    - required alignment = is a num that describe where to allocate a data in memory in order to be valid, that valid address most be divisble by that number(required alignment)
    - example usage: "data have alignment of 8 bytes(a valid address for this data are divible by 8)"
- each type have different alignment (i thought why just make them all have alignment 1, so you can place any data in any address. But that will dencrease CPU efficiency as i know(need search))
- struct alignment is largetst field alignment ((why?, need search))
- The size of a value is always a multiple of its alignment

- `padding` is an ignored address(empty) within the arena (that was being allocated), it contains a garbage value or if initialize 0 value
    - less padding = more performance(need search)
