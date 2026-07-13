# arenas-rs
Building arenas (region-based) memorey allocators in [rust](https://rust-lang.org/), implementing bump, stack, poll and free-list allocators

# Concepts
> [!NOTE]
> most of the time when i say "allocate" i meant to allocate some block "inside" the region/arena
>
> i didn't impl any fancy data structure or algorithms to build allocator(e.g., linked-list), just simple naive approach

## general purpose allocator
it is the simpliest allocator, it just allocates or deallactes memory, no fancy methods or algorithms, in c or rust, it is the default allocator
## Bump/leaner allocator
- it basically can allocate memory leanery only in one direction, that's means you can't like deallocate prev allocation.
> [!NOTE]
> if you want an allocator that deallocate memory on the last allocation on the arena, consider reading about the next allocator [the stack allocator](#Stack-allocator)

- if you want deallocation, you'll need to deallocate all the region/arena all in once (that's where the name "bump" came from!)
### implimentation 
- `build` method: build the arena with a specific layout
- `push`  method: push data to the arena, returning a pointer to the start of it
- `clear` method: deallocate the arena
### what you gonna need ?
- a "layout" to know the size of the arena(we only using size of the layout, won't using alignemnt cuz its 1), and also needed as input for alloc/dealloc functions
- a "pointer" points to the beginning of the arena, in order to deallocate(bumping) it later with dealloc() function
- a "tracker pointer" that tracks where the "end of last data allocation pos", and tracks requested data start pos
- "used-bytes-counter" that tracks "how many bytes" that are allocated to check if you can allocate more data by comparing it with "arena size", "requested-bytes" and padding(comes from aligning that data, which gives a data valid memory address)
### challenges
- the only challenge you'll face is offseting the tracker pointer for the next allocation,you need first to align current pointer for the requested-data alignment, then add to the tracker requasted-data size, lukcy for us rust does have a method for aligning data "align_offset()" method on a pointer type (our pointer tracker), so you need just to add requested-data size
> [!NOTE]
> if you want to align the data without rust's method, you only need the modular of current used bytes by data alignment, then use it to substract data alignment with it
### Resources
https://www.gingerbill.org/article/2019/02/08/memory-allocation-strategies-002/

## Stack allocator
- It's just like the leaner allocator, the only difference is poll allocator stores metadata of every tracker that comes from pushing data to the arena, and uses-bytes between each offset in order to pop the last allocated data
> [!NOTE]
> you might ask why we can don't impl removing from different location rather the the alst one , I would answer by there's another allocator that does what you ask but with tiny minor changes [the pool allocator](#Pool-allocator)
### implimentation
- `build` method: build the arena with a specific layout, with initializing previous tracker beside previous size allocation metadata of the stack allocator struct
- `push` method: push data to the arena, returning a pointer to the start of it, storing tracker before the offseting and size of allocation (including padding)
- `pop` method: pop the last allocation from the arena using prev tracker and prev size allocation
- `clear` method: to deallocate the arena
### what you gonna need ?
- same step as the bump allocator
- the only addition thing is to store prev tracker and prev allocation size in order to optionnaly pop it later
### Resources
https://www.gingerbill.org/article/2019/02/08/memory-allocation-strategies-003/
## Pool allocator
- this alloactor can allocate objects randomly with defined blocks that have the same layout (with size = alignment) inside the arena, it will additionnaly stores layout,tracker and bool if it free to use as a Block strcut as metadata to pool allocator. The pool allocator only have trailling padding, cuz when building the allocator, i already build the blocks(allocate), i just need to change what inside those blocks(data)

### implimentation
- `build` method: build arena with specific layout and block size/alignment, constructing arena and tracker for each block
- `push`  method: push data to arena by searching for available/free block, returing a pointer to that block
- `remove` method: get a tracker to a available block. make that block free to use
- `clear` method: deallocate the arena
### Resources
https://www.gingerbill.org/article/2019/02/16/memory-allocation-strategies-004/
# What i learned (notes on memory concepts, author's notes), this section answored some question i have on memory)

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
    - padding can be also when the size of struct/enum is not multiple of its alignment, that padding at the end called `trailled padding` (rust will handle that, the struct/enum's size will be multiple of it's alignment)

- `memory currpetion` means a pointer writes to or reads from memory it shouldn't (can be inside or outside the allocation), can lead to undefined behavior
- `memory leak` means a memory was allocated but never release, if that happen regularly, will runs out of memory
- `fragmentation` means free memory (padding) is unusable inside the allocation that cuzed by bad layout alignemnt
- `fragmentation fault` or `core dumb` is an error that occur when a program try to access memory from other program

# What i learnt (notes on other then memory concepts, this section is heavliy for me)
- don't make a function when the body is one ( i make logging with only one line of code instead of just write that line on the caller function)
