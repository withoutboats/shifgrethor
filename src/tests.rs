use super::*;

#[test]
fn heap_rooted() {
    let _ = env_logger::try_init();
    // It is in fact a pointer to the value
    let ptr1 = HeapRoot::new(0xBEEFDADi32);
    let addr1 = &*ptr1 as *const i32 as usize;
    assert_eq!(*ptr1, 0xBEEFDAD);

    // Running the collector does not collect it, because it is still alive
    collect();
    println!("here");
    assert_eq!(*ptr1, 0xBEEFDAD);

    // Create a second copy it
    let ptr2 = ptr1.clone();

    // Assert that they point to the same object
    assert_eq!(*ptr2, 0xBEEFDAD);
    let addr2 = &*ptr2 as *const i32 as usize;
    assert_eq!(addr1, addr2);

    // drop ptr1
    drop(ptr1);

    // Ensure that the object is still rooted and not collected
    collect();
    assert_eq!(*ptr2, 0xBEEFDAD);
    drop(ptr2);

    // Ensure that it gets collected once all roots are gone
    collect();
    assert_eq!(raw::count_managed_objects(), 0);
}

#[test]
fn stack_rooted() {
    let _ = env_logger::try_init();
    letroot!(root);
    let ptr1 = root.gc(0xBADCAFE);
    assert_eq!(*ptr1, 0xBADCAFE);
    let ptr2 = ptr1;
    assert_eq!(*ptr2, 0xBADCAFE);
}

#[test]
fn rerooting() {
    letroot!(root);

    let (addr, x2): (usize, HeapRoot<i32>) = {
        letroot!(inner);
        let x1 = inner.gc(0);
        collect();
        assert_eq!(*x1, 0);
        (&*x1 as *const i32 as usize, HeapRoot::reroot(x1))
    };

    collect();
    assert_eq!(*x2, 0);
    assert_eq!(&*x2 as *const i32 as usize, addr);

    let x3 = root.reroot(x2.gc());
    drop(x2);

    collect();
    assert_eq!(*x3, 0);
    assert_eq!(&*x3 as *const i32 as usize, addr);
}
