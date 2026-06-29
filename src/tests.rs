use super::*;
const LENGTH: usize = 1000;

/// bumb allocator tests
#[test]
fn arena_size() {
    let bumb_alloc = BumbAlloc::build(LENGTH);

    assert_eq!(bumb_alloc.arena.layout.size(), LENGTH);
}

#[test]
fn not_enough_space_to_allocate() {
    let mut bumb_alloc = BumbAlloc::build(0);
    let layout = Layout::new::<i32>();
    let ptr = bumb_alloc.push(&layout);
    // not enough space, arena.arena.used_bytes will not change
    assert_eq!(bumb_alloc.arena.used_bytes, 0);
    assert!(ptr.is_null());
}

#[test]
fn bumb_alloc() {
    let mut bumb_alloc = BumbAlloc::build(LENGTH);

    let layout = Layout::new::<i32>();
    let _ = bumb_alloc.push(&layout);

    bumb_alloc.clear();

    assert_eq!(bumb_alloc.arena.layout.size(), 0);
    // can't have alignment of 0 bytes in rust
    assert_eq!(bumb_alloc.arena.layout.align(), 1);
    assert_eq!(bumb_alloc.arena.used_bytes, 0);
    assert_eq!(bumb_alloc.arena.tracker, ptr::null_mut());
    assert_eq!(bumb_alloc.arena.start, ptr::null_mut());
}

#[test]
fn arena_push_primitives() {
    let mut bumb_alloc = BumbAlloc::build(LENGTH);

    let layout = Layout::new::<[i16; 2]>();
    let ptr_1 = bumb_alloc.push(&layout).cast::<[i16; 2]>();
    assert_eq!(bumb_alloc.arena.used_bytes, 4);

    let layout = Layout::new::<[i64; 3]>();
    let ptr_2 = bumb_alloc.push(&layout).cast::<[i64; 3]>();
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
fn arena_push_struct() {
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
    let mut bumb_alloc = BumbAlloc::build(LENGTH);

    let ptr_1 = bumb_alloc.push(&foo_layout).cast::<Foo>();
    assert_eq!(bumb_alloc.arena.used_bytes, foo_layout.size());

    let ptr_2 = bumb_alloc.push(&boo_layout).cast::<Boo>();
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

// Stack allocator tests
#[test]
fn arena_pop() {
    let mut stack_alloc = StackAlloc::build(32);

    // with pirimitives
    let layout = Layout::new::<i32>();
    let ptr_1 = stack_alloc.push(&layout);
    let _ptr_2 = stack_alloc.push(&layout); // will get dropped
    stack_alloc.pop(); // ptr_2 is invalid
    assert_eq!(stack_alloc.arena.used_bytes, 4);
    assert_eq!(stack_alloc.arena.tracker, unsafe {
        stack_alloc.arena.start.add(4)
    });

    // with structs
    #[derive(Debug)]
    struct Foo {
        data: u64,
        some_data: &'static str,
    }
    let _ptr_2 = stack_alloc.push(&layout);
    let _ptr_3 = stack_alloc.push(&layout); // will get dropped
    stack_alloc.pop(); // ptr_3 is invalid
    let layout = Layout::new::<Foo>();
    let ptr_2 = stack_alloc.push(&layout).cast::<Foo>();
    let arena_used_bytes = 8;
    let struct_size = 24;
    assert_eq!(stack_alloc.arena.used_bytes, arena_used_bytes + struct_size);
    let arena_start = stack_alloc.arena.start;
    assert_eq!(stack_alloc.arena.tracker, unsafe {
        arena_start.add(arena_used_bytes + struct_size)
    });

    // check if pointer are valid
    unsafe {
        ptr_1.write(67);
        ptr_2.write(Foo {
            data: 123,
            some_data: "hello world",
        });
        dbg!(ptr_1);
        dbg!(ptr_2);
    }
}
