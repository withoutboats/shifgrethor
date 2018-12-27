use super::*;

#[test]
fn collect_does_not_affect_reachable_roots() {
    // At this point there should be no roots
    assert_eq!(raw::count_roots(), 0);

    letroot!(_root); // Create a root on the stack

    assert_eq!(raw::count_roots(), 1);

    // Even if we try to call collect() the root must live
    //  because it lives on the stack and the current scope has not ended yet.
    collect();
    assert_eq!(raw::count_roots(), 1);
}

#[test]
fn count_roots_create_incr_destroy_decr() {
    assert_eq!(raw::count_roots(), 0);

    {   letroot!(_root); // Create a root on the stack

        // Once we created a root there should be exactly 1 root
        assert_eq!(raw::count_roots(), 1);

    } // We left the scope where the root was created

    // The root is out of scope and destroyed
    assert_eq!(raw::count_roots(), 0);
}

#[test]
fn count_roots_multiple_in_one_scope() {
    assert_eq!(raw::count_roots(), 0);

    {

        letroot!(_root1);
        assert_eq!(raw::count_roots(), 1);

        letroot!(_root2);
        assert_eq!(raw::count_roots(), 2);

    }

    assert_eq!(raw::count_roots(), 0);
}

#[test]
fn count_roots_multiple_in_inherit_scope() {
    assert_eq!(raw::count_roots(), 0);

    {   letroot!(_root1);
        assert_eq!(raw::count_roots(), 1);

        {   letroot!(_root2);

            assert_eq!(raw::count_roots(), 2);

        }

        assert_eq!(raw::count_roots(), 1);

    }

    assert_eq!(raw::count_roots(), 0);
}

#[test]
fn stack_rooted() {
    let _ = env_logger::try_init();

    {   letroot!(root);

        let ptr1 = root.gc(0xBADCAFE);
        assert_eq!(*ptr1, 0xBADCAFE);
        collect();
        let ptr2 = ptr1;
        assert_eq!(*ptr2, 0xBADCAFE);

    }

    // Ensure that it gets collected once all roots are gone
    collect();
    assert_eq!(raw::count_managed_objects(), 0);
}

#[test]
fn rerooting() {
    let _ = env_logger::try_init();

    {   letroot!(outer_root);

        let ptr2 = {   letroot!(inner_root);

            // It is in fact a pointer to the value
            let ptr1 = inner_root.gc(0xBEEFDAD);
            let addr1 = &*ptr1 as *const i32 as usize;
            assert_eq!(*ptr1, 0xBEEFDAD);

            // Running the collector does not collect it, because it is still alive
            collect();
            println!("here");
            assert_eq!(*ptr1, 0xBEEFDAD);

            // Create a second copy it
            let ptr2 = outer_root.reroot(ptr1);

            // Assert that they point to the same object
            assert_eq!(*ptr2, 0xBEEFDAD);
            let addr2 = &*ptr2 as *const i32 as usize;
            assert_eq!(addr1, addr2);
            ptr2
        };

        // Ensure that the object is still rooted and not collected
        collect();
        assert_eq!(*ptr2, 0xBEEFDAD);

    }

    // Ensure that it gets collected once all roots are gone
    collect();
    assert_eq!(raw::count_managed_objects(), 0);
}
