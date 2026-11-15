use crate::arenas::{
    arena::ArenaError,
    bump_allocator::BumpAlloc,
    pool_allocator::{PoolAlloc, PoolAllocError},
    stack_allocator::StackAlloc,
};
use std::alloc::Layout;

// pool allocator
#[test]
fn pool_allocator_blocks_tracker() {
    const LENGTH: usize = 10;
    const BLOCK_SIZE: usize = 2;
    let pool_alloc = PoolAlloc::build(LENGTH, BLOCK_SIZE).unwrap();
    let ptr_start = pool_alloc.arena.start.unwrap();

    assert_eq!(pool_alloc.blocks[0].tracker, ptr_start);
    unsafe {
        assert_eq!(pool_alloc.blocks[1].tracker, ptr_start.add(BLOCK_SIZE));
        assert_eq!(pool_alloc.blocks[2].tracker, ptr_start.add(BLOCK_SIZE * 2));
    }
}

#[test]
fn pool_alloc_pointer_update() {
    const BLOCK_SIZE: usize = 32;
    const POOL_ALLOC_SIZE: usize = 100;
    let mut pool_alloc = PoolAlloc::build(POOL_ALLOC_SIZE, BLOCK_SIZE).unwrap();
    let layout = Layout::new::<i16>();

    let ptr_start = pool_alloc.arena.tracker.unwrap();
    let _ = pool_alloc.push(&layout).unwrap();
    let ptr_update_1 = pool_alloc.push(&layout).unwrap();
    let ptr_update_2 = pool_alloc.push(&layout).unwrap();

    unsafe {
        assert_eq!(ptr_start.add(BLOCK_SIZE), ptr_update_1);
        assert_eq!(ptr_start.add(BLOCK_SIZE * 2), ptr_update_2);
    }
}

#[test]
fn pool_alloc_push_all_blocks_being_used() {
    const LENGTH: usize = 4;
    let mut pool_alloc = PoolAlloc::build(LENGTH, 2).unwrap();
    let layout = Layout::new::<i16>();
    pool_alloc.push(&layout).unwrap();
    pool_alloc.push(&layout).unwrap();
    let pointer = pool_alloc.push(&layout);

    // all blocks being used
    assert_eq!(pointer.unwrap_err(), PoolAllocError::PoolAllocFull);
}

#[test]
fn pool_alloc_push_an_instance_is_bigger_then_block_size() {
    const LENGTH: usize = 4;
    let mut pool_alloc = PoolAlloc::build(LENGTH, 2).unwrap();
    let layout = Layout::new::<i32>();
    let pointer = pool_alloc.push(&layout);

    // instance's size is bigger then block's size
    assert_eq!(pointer.unwrap_err(), PoolAllocError::OutOfBoundRequest);
}

#[test]
fn pool_allocator_remove() {
    const LENGTH: usize = 5;
    let mut pool_alloc = PoolAlloc::build(LENGTH, 2).unwrap();

    let layout = Layout::new::<i16>();
    let pointer_1 = pool_alloc.push(&layout).unwrap();
    let _pointer_2 = pool_alloc.push(&layout).unwrap();

    unsafe {
        assert_eq!(
            pool_alloc.blocks[0].tracker,
            pool_alloc.arena.start.unwrap()
        );
        assert_eq!(
            pool_alloc.blocks[1].tracker,
            pool_alloc.arena.start.unwrap().add(2)
        );
    }

    pool_alloc.remove(pointer_1).unwrap();
    assert!(pool_alloc.blocks[0].is_used == false);
}

// bumb/arena allocator tests
#[test]
fn bumb_alloc_size() {
    const LENGTH: usize = 1000;
    let bumb_alloc = BumpAlloc::build(LENGTH).unwrap();

    assert_eq!(bumb_alloc.arena.layout.size(), LENGTH);
}

#[test]
fn bumb_alloc_not_enough_space_to_allocate() {
    let mut bumb_alloc = BumpAlloc::build(0).unwrap();
    let layout = Layout::new::<i32>();
    let ptr = bumb_alloc.push(&layout);
    assert_eq!(ptr.unwrap_err(), ArenaError::ArenaFull);
    assert_eq!(bumb_alloc.arena.used_bytes, 0);
}

#[test]
fn bump_alloc_pointer_update() {
    let mut bumb_alloc = BumpAlloc::build(100).unwrap();
    let layout = Layout::new::<i32>();
    let ptr_start = bumb_alloc.arena.tracker.unwrap();
    let _ = bumb_alloc.push(&layout).unwrap();
    let ptr_update_1 = bumb_alloc.push(&layout).unwrap();
    let ptr_update_2 = bumb_alloc.push(&layout).unwrap();
    unsafe {
        assert_eq!(ptr_start.add(size_of::<i32>()), ptr_update_1);
        assert_eq!(ptr_start.add(size_of::<[i32; 2]>()), ptr_update_2);
    }
}

#[test]
fn bumb_alloc_push_primitives() {
    const LENGTH: usize = 1000;
    let mut bumb_alloc = BumpAlloc::build(LENGTH).unwrap();

    let layout = Layout::new::<[i16; 2]>();
    let ptr_1 = bumb_alloc.push(&layout).unwrap().cast::<[i16; 2]>();
    assert_eq!(bumb_alloc.arena.used_bytes, 4);

    let layout = Layout::new::<[i64; 3]>();
    let ptr_2 = bumb_alloc.push(&layout).unwrap().cast::<[i64; 3]>();
    let padding = 4;
    let prev_area_used_bytes = 4;
    assert_eq!(
        bumb_alloc.arena.used_bytes,
        prev_area_used_bytes + layout.size() + padding
    );

    // testing bumb_alloc.arena.tracker
    unsafe {
        ptr_1.write([1, 2]);
        ptr_2.write([3, 4, 5]);
        assert_eq!(ptr_1.read(), [1, 2]);
        assert_eq!(ptr_2.read(), [3, 4, 5]);
    }

    bumb_alloc.clear();
}

#[test]
fn bumb_alloc_push_struct() {
    const LENGTH: usize = 1000;

    #[derive(Debug)]
    struct Foo {
        data: bool,
        more_data: u64,
    }
    #[derive(Debug)]
    struct Boo {
        some_data: u8,
        more_more_data: char,
    }

    let foo_layout = Layout::new::<Foo>();
    let boo_layout = Layout::new::<Boo>();
    let mut bumb_alloc = BumpAlloc::build(LENGTH).unwrap();

    let ptr_1 = bumb_alloc.push(&foo_layout).unwrap().cast::<Foo>();
    assert_eq!(bumb_alloc.arena.used_bytes, foo_layout.size());

    let ptr_2 = bumb_alloc.push(&boo_layout).unwrap().cast::<Boo>();
    let prev_area_used_bytes = 16;
    assert_eq!(
        bumb_alloc.arena.used_bytes,
        prev_area_used_bytes + boo_layout.size()
    );

    unsafe {
        ptr_1.write(Foo {
            data: true,
            more_data: 2837,
        });

        ptr_2.write(Boo {
            some_data: 255,
            more_more_data: '1',
        });

        dbg!(ptr_1);
        dbg!(ptr_1.read());
        dbg!(ptr_2);
        dbg!(ptr_2.read());
    }
    bumb_alloc.clear();
}

#[test]
fn use_tracker_start_on_empty_arena() {
    const LENGTH: usize = 1000;
    let mut bumb_alloc = BumpAlloc::build(LENGTH).unwrap();

    bumb_alloc.clear();

    assert!(bumb_alloc.arena.start.is_none());
}

//Stack allocator tests
#[test]
fn stack_alloc_pop() {
    let mut stack_alloc = StackAlloc::build(100).unwrap();

    // with pirimitives
    let layout = Layout::new::<i32>();
    let _ptr_1 = stack_alloc.push(&layout).unwrap();
    let layout = Layout::new::<i128>();
    let _ptr_2 = stack_alloc.push(&layout).unwrap();
    let layout = Layout::new::<i8>();
    let _ptr_3 = stack_alloc.push(&layout).unwrap();
    stack_alloc.pop().unwrap();
    dbg!(&stack_alloc);
    stack_alloc.pop().unwrap();
    dbg!(&stack_alloc);
    stack_alloc.pop().unwrap();
    dbg!(&stack_alloc);
    let error = stack_alloc.pop().unwrap_err();
    assert_eq!(error, ArenaError::ArenaEmpty);
    dbg!(&stack_alloc);
    assert_eq!(stack_alloc.arena.used_bytes, 0);
}

#[test]
fn stack_alloc_pointer_update() {
    let mut stack_alloc = StackAlloc::build(100).unwrap();
    let layout = Layout::new::<i32>();
    let ptr_start = stack_alloc.arena.tracker.unwrap();
    let _ = stack_alloc.push(&layout).unwrap();
    let ptr_update_1 = stack_alloc.push(&layout).unwrap();
    let ptr_update_2 = stack_alloc.push(&layout).unwrap();
    unsafe {
        assert_eq!(ptr_start.add(size_of::<i32>()), ptr_update_1);
        assert_eq!(ptr_start.add(size_of::<[i32; 2]>()), ptr_update_2);
    }
}
