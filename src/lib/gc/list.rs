use std::cell::Cell;
use std::marker::{Pinned, PhantomData};
use std::pin::Pin;
use std::ptr::NonNull;

pub struct List<T: AsRef<List<T>> + ?Sized> {
    prev: Cell<Option<NonNull<List<T>>>>,
    next: Cell<Option<NonNull<T>>>,
    _pinned: Pinned,
}

impl<T: AsRef<List<T>> + ?Sized> Default for List<T> {
    fn default() -> List<T> {
        List {
            prev: Cell::default(),
            next: Cell::default(), 
            _pinned: Pinned,
        }
    }
}

impl<T: AsRef<List<T>> + ?Sized> List<T> {
    pub fn insert(self: Pin<&Self>, new: Pin<&T>) {
        let this: &Self = &*self;
        let new: &T = &*new;

        let list: &List<T> = new.as_ref();
        list.prev.set(Some(NonNull::from(this)));
        list.next.set(this.next.get());

        if let Some(next) = this.next.get() {
            unsafe {
                let next: &List<T> = next.as_ref().as_ref();
                next.prev.set(Some(NonNull::from(list)));
            }
        }

        this.next.set(Some(NonNull::from(new)));
    }

    pub fn is_head(&self) -> bool {
        self.prev.get().is_none()
    }
}

impl<T: AsRef<List<T>> + ?Sized> Drop for List<T> {
    fn drop(&mut self) {
        if let Some(prev) = self.prev.get() {
            unsafe { prev.as_ref().next.set(self.next.get()); }
        }
        if let Some(next) = self.next.get() {
            unsafe { next.as_ref().as_ref().prev.set(self.prev.get()); }
        }
    }
}

impl<'a, T: AsRef<List<T>> + ?Sized> IntoIterator for Pin<&'a List<T>> {
    type IntoIter = Iter<'a, T>;
    type Item = Pin<&'a T>;
    fn into_iter(self) -> Iter<'a, T> {
        Iter {
            next: (*self).next.get(),
            _marker: PhantomData,
        }
    }
}

pub struct Iter<'a, T: AsRef<List<T>> + ?Sized + 'a> {
    next: Option<NonNull<T>>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: AsRef<List<T>> + ?Sized> Iterator for Iter<'a, T> {
    type Item = Pin<&'a T>;
    fn next(&mut self) -> Option<Pin<&'a T>> {
        if let Some(next) = self.next {
            unsafe {
                self.next = next.as_ref().as_ref().next.get();
                Some(Pin::new_unchecked(&*next.as_ptr()))
            }
        } else {
            None
        }
    }
}
