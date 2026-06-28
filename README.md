# arenas-rs
Building arenas (region-based) memorey allocators in rust, implementing bumb, stack, free-list and pool allocator

> [!NOTE]
> most of the time when i say "allocate" i meant to allocate some block "inside" the region/arena

# Concepts
## general purpose allocator
it is the simpliest allocator, it just allocates or deallactes memory, no fancy methods or algorithms, in c or rust, it is the default allocator
## Bumb/leaner allocator
- it basically can allocate memory leanery only in one direction, that's means you can't like deallocate prev allocation.
> [!NOTE]
> if you want an allocator that deallocate memory on the last allocation on the arena, consider reading about the next allocator (the stack allocator)

- if you want deallocation, you'll need to deallocate all the region/arena all in once (that's where the name "bumb" came from!)
### implimentation 
- we gonna impl
    - `build` function: to build the arena with specific layout
    - `push`  function: to push data to the arena, returning a pointer to the start of it
    - `clear` function: to deallocate the arena
#### what you gonna need ?
- a "layout" know the size of the arena(we only using size of the layout, won't using alignemnt cuz its 1), and also needed for dealloc () function
- a "pointer" points to the beginning of the arena, in order to deallocate(bumbing) it later with dealloc() function
- a "tracker pointer" that tracks where the "end of last data allocation pos", and tracks requested data start pos
- "used bytes counter" that tracks "how many bytes" that are allocated to check if you can allocate more size, by comparing it with "arena size" and "requested-bytes"
#### challenges
- the only challenge you'll face is aligning data (give data valid memory address that is based on the data alignment), you gonna offset the pointer by adding padding (if needed) that are result of aligning data, and by adding also data size to the offset value for the pointer, lukcy for us rust does have a method for aligning data "align_offset" method. for increamenting used bytes value, it equals to the offset value, cuz we are not restrict to arena alignemnt, which equals to 1
> [!NOTE]
> if you want to align the data, without rust method, you only need the modular of current used bytes by data alignment, that used to substract data alignment with it
### Resources
https://www.gingerbill.org/article/2019/02/08/memory-allocation-strategies-002/

# What i learned (Author's notes)

- each block in memroy represent an address, we deal with addresses as bytes, and hexdecimal number
- rust std returend `u8`(a byte in size) as a pointee type, make pointer manipulation easy to resaon about cuz your are doing them with pure bytes

- a `layout` is a description/request of a chunk of memory(no allocation yet), that request a size and an alignment
    - the required alignment is just an information, you'll impl the offset of the pointer manually to make an aligned data, the length is need for allocation

- `memory alignment` describes where piece of data should store in memory, a data is properly aligned if its "starting address" is a "multiple" of its "required alignemnt", when this condition satisfied, we say "the data is "aligned" or "properley aligned""
    - required alignment = is a num that describe where to allocate a data in memory in order to be valid, that valid address most be divisble by that number(required alignment)
    - example usage: "data have alignment of 8 bytes(a valid address for this data are divible by 8)"
    - "align memory address" give it valid memory address based on the memory alignment
- each type have different alignment (i thought why just make them all have alignment 1, so you can place any data in any address. But that will dencrease CPU efficiency as i know(search))
- struct alignment is the largetst field alignment ((why?, search))
- The size of a value is always a multiple of its alignment

- `padding` is an ignored address(empty) within the arena, it contains a garbage value or if initialized 0 or \0 value
    - less padding = more performance(search)
    - in order to align a data, you need to offset the pointer forward, you'll have padding between the prev and current pos of the pointer
    - 1 padding = 1 byte
    - padding between allocation do count/add as/to arena's used bytes, but they didn't count as used space( no data owns it)

- `memory currpetion` means a pointer writes to or reads from memory it shouldn't (can be inside or outside the allocation), can lead to undefined behavior
- `memory leak` means a memory was allocated but never release, if that happen regularly, will runs out of memory
- `fragmentation` means free memory (padding) is unusable inside the allocation that cuzed by bad layout alignemnt
