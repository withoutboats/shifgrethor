use std::marker::{PhantomData, PhantomPinned};

use gc::{GcPtr, Trace};

use crate::Gc;

pub struct GcStore<'root, T: ?Sized + 'root> {
    ptr: GcPtr<T>,
    _marker: PhantomData<(&'root T, PhantomPinned)>,
}

impl<'root, T: Trace> GcStore<'root, T> {
    pub fn new(data: T) -> GcStore<'root, T> {
        GcStore {
            ptr: gc::alloc_unmanaged(data),
            _marker: PhantomData,
        }
    }
}

impl<'root, T: ?Sized> GcStore<'root, T> {
    pub fn get(&self) -> &T {
        unsafe {
            if self.ptr.is_unmanaged() {
                self.ptr.data()
            } else {
                panic!("Cannot call `GcStore::get` after the GcStore has been rooted.")
            }
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        panic!()
    }

    pub fn get_maybe(&self) -> Option<&T> {
        unsafe {
            if self.ptr.is_unmanaged() {
                Some(self.ptr.data())
            } else {
                None
            }
        }
    }

    pub fn get_mut_maybe(&mut self) -> Option<&mut T> {
        panic!()
    }

    pub fn raw(this: &GcStore<'root, T>) -> GcPtr<T> {
        this.ptr
    }
}

unsafe impl<'root, T: Trace + ?Sized> Trace for GcStore<'root, T> {
    unsafe fn mark(&self) {
        self.ptr.mark();
    }

    unsafe fn manage(&self) {
        self.ptr.manage();
    }

    unsafe fn finalize(&mut self) { }
}

impl<'root, T: ?Sized + Trace> From<Gc<'root, T>> for GcStore<'root, T> {
    fn from(gc: Gc<'root, T>) -> GcStore<'root, T> {
        GcStore {
            ptr: Gc::raw(gc),
            _marker: PhantomData,
        }
    }
}

impl<'root, T: ?Sized> Drop for GcStore<'root, T> {
    fn drop(&mut self) {
        unsafe {
            if self.ptr.is_unmanaged() {
                self.ptr.deallocate()
            }
        }
    }
}
