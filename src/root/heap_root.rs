use std::ops::Deref;
use std::pin::Pin;

use gc::{GcPtr, Root, Trace};

use crate::Gc;
use crate::root::Reroot;

pub struct HeapRoot<T: ?Sized> {
    #[allow(dead_code)]
    root: Pin<Box<Root>>,
    ptr: GcPtr<T>,
}

impl<'root, T> HeapRoot<T> where
    T: Reroot<'root> + Trace,
    T::Rerooted: Trace,
{
    pub fn new(data: T) -> HeapRoot<T::Rerooted> {
        unsafe {
            HeapRoot::make(gc::alloc(data))
        }
    }
}

impl<'root, T> HeapRoot<T> where
    T: Reroot<'root> + ?Sized,
    T::Rerooted: Trace,
{
    pub fn reroot(gc: Gc<'_, T>) -> HeapRoot<T::Rerooted> {
        unsafe {
            HeapRoot::make(Gc::raw(gc))
        }
    }
}

impl<'root, T> HeapRoot<T> where
    T: Reroot<'root> + ?Sized,
    T::Rerooted: Trace,
{
    unsafe fn make(ptr: GcPtr<T>) -> HeapRoot<T::Rerooted> {
        let ptr = super::reroot(ptr);
        let root = Pin::from(Box::new(Root::new(ptr)));
        gc::enroot(root.as_ref());
        HeapRoot { root, ptr }
    }
}

impl<T: ?Sized> HeapRoot<T> {
    pub fn gc<'root>(&'root self) -> Gc<'root, T> {
        unsafe {
            Gc::rooted(self.ptr)
        }
    }
}

impl<T: Trace + ?Sized> Clone for HeapRoot<T> {
    fn clone(&self) -> HeapRoot<T> {
        let root = unsafe {
            Pin::from(Box::new(Root::new(self.ptr)))
        };
        gc::enroot(root.as_ref());
        HeapRoot {
            root,
            ptr: self.ptr,
        }
    }
}

impl<T: ?Sized> Deref for HeapRoot<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            self.ptr.data()
        }
    }
}
