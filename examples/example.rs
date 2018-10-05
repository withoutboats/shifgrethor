#![feature(arbitrary_self_types)]

use shifgrethor::{Gc, GcStore, letroot, GC, Finalize};

#[derive(GC)]
#[gc(finalize)]
struct Foo<'root> {
    int: u64,
    #[gc] bar: GcStore<'root, Bar<'root>>,
}

impl<'root> Foo<'root> {
    pub fn new(int: u64, data: String) -> Foo<'root> {
        Foo {
            int: int,
            bar: GcStore::new(Bar::new(data)),
        }
    }

    pub fn gc_method(self: Gc<'root, Foo<'root>>, x: u64) -> u64 {
        let bar: Gc<Bar> = self.bar();
        let data: Gc<String> = bar.data();
        println!("{}", &*data);
        self.int + x
    }
}

impl<'root> Finalize for Foo<'root> {
    fn finalize(&mut self) {
        println!("Dropping ({})", self.int);
    }
}

#[derive(GC)]
struct Bar<'root> {
    #[gc] data: GcStore<'root, String>,
}

impl<'root> Bar<'root> {
    pub fn new(data: String) -> Bar<'root> {
        Bar {
            data: GcStore::new(data),
        }
    }
}

fn main() {
    {
        let foo = Foo::new(2, String::from("Hello, world!"));
        letroot!(root);
        let foo = root.gc(foo);
        shifgrethor::collect();
        println!("{}", foo.gc_method(2));
    }
    shifgrethor::collect();
}
