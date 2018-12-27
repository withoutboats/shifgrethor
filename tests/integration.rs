use shifgrethor::{letroot, collect, GC, Finalize, raw};

#[test]
fn collect_recursive() {
    #[derive(GC)]
    #[gc(finalize)]
    struct Getheren {
    }

    impl Getheren {
        pub fn new() -> Getheren {
            Getheren { }
        }
    }

    impl Finalize for Getheren {
        fn finalize(&mut self) {
            collect(); // Now nameless I will go seek my death.
        }
    }

    {   letroot!(root);

        let _getheren = root.gc(Getheren::new());
        assert_eq!(raw::count_managed_objects(), 1);
    }

    collect();
    assert_eq!(raw::count_managed_objects(), 0);
}
