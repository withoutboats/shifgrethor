use std::ops::{Deref, DerefMut};

use gc::Trace;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct NoTrace<T: ?Sized> {
    data: T,
}

impl<T> NoTrace<T> {
    pub unsafe fn new_unchecked(data: T) -> NoTrace<T> {
        NoTrace { data } 
    }
}

impl<T: 'static> NoTrace<T> {
    pub fn new_static(data: T) -> NoTrace<T> {
        NoTrace { data } 
    }
}

impl<T: Copy> NoTrace<T> {
    pub fn new_copy(data: T) -> NoTrace<T> {
        NoTrace { data } 
    }

}

impl<T: ?Sized> Deref for NoTrace<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T: ?Sized> DerefMut for NoTrace<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

unsafe impl<T: ?Sized> Trace for NoTrace<T> {
    unsafe fn mark(&self) { }
    unsafe fn manage(&self) { }
    unsafe fn finalize(&mut self) { }
}
