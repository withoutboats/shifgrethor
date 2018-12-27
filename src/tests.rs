use super::*;


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
