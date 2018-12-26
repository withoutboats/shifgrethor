use std::fmt;
use std::hash;
use std::marker::{PhantomData, PhantomPinned};
use std::ops::Deref;
use std::pin::Pin;

use gc::{GcPtr, Trace};

pub struct Gc<'root, T: ?Sized + 'root> {
    ptr: GcPtr<T>,
    _marker: PhantomData<(&'root T, PhantomPinned)>,
}

impl<'root, T: ?Sized> Clone for Gc<'root, T> {
    fn clone(&self) -> Gc<'root, T> {
        *self
    }
}

unsafe impl<'root, T: Trace + ?Sized> Trace for Gc<'root, T> {
    unsafe fn mark(&self) { }

    unsafe fn manage(&self) { }

    unsafe fn finalize(&mut self) { }
}

impl<'root, T: ?Sized> Gc<'root, T> {
    pub unsafe fn rooted(ptr: GcPtr<T>) -> Gc<'root, T> {
        Gc { ptr,
            _marker: PhantomData,
        }
    }

    // NOTE: Problematic for copying collectors
    pub fn pin(self) -> Pin<Gc<'root, T>> {
        unsafe {
            Pin::new_unchecked(self)
        }
    }

    pub fn raw(this: Gc<'root, T>) -> GcPtr<T> {
        this.ptr
    }
}

impl<'root, T: ?Sized> Deref for Gc<'root, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            self.ptr.data()
        }
    }
}

impl<'root, T: fmt::Debug + ?Sized> fmt::Debug for Gc<'root, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &*self;
        write!(f, "Gc({:?})", inner)
    }
}

impl<'root, T: fmt::Display + ?Sized> fmt::Display for Gc<'root, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        T::fmt(&*self, f)
    }
}

impl<'root, T: ?Sized> Copy for Gc<'root, T> { }

impl<'root, T: PartialEq + ?Sized> PartialEq for Gc<'root, T> {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe {
            self.ptr.data() == rhs.ptr.data()
        }
    }
}

impl<'root, T: Eq + ?Sized> Eq for Gc<'root, T> { }

impl<'root, T: PartialOrd + ?Sized> PartialOrd for Gc<'root, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe {
            self.ptr.data().partial_cmp(other.ptr.data())
        }
    }
}

impl<'root, T: Ord + ?Sized> Ord for Gc<'root, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe {
            self.ptr.data().cmp(other.ptr.data())
        }
    }
}

impl<'root, T: hash::Hash + ?Sized> hash::Hash for Gc<'root, T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        unsafe {
            self.ptr.data().hash(state)
        }
    }
}
