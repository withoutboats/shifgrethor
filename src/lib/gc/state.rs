use std::cell::{Ref, RefCell};
use std::pin::Pin;
use std::ptr::NonNull;

use log::*;

use crate::alloc::{Allocation, Data};
use crate::gc_ptr::GcPtr;
use crate::list::List;
use crate::trace::Trace;

#[derive(Default)]
pub struct GcState {
    objects: List<Allocation<Data>>,
    roots: RefCell<Vec<NonNull<Allocation<Data>>>>,
}

impl GcState {
    pub fn collect(self: Pin<&Self>) {
        for root in &self.roots()[..] {
            debug!("TRACING from root at:       {:x}", &*root as *const _ as usize);
            unsafe {
                root.as_ref().mark();
            }
        }

        for object in self.objects() {
            if !object.marked() {
                debug!("FREEING unmarked object at: {:x}", &*object as *const _ as usize);
                unsafe {
                    (&*object as *const Allocation<Data> as *mut Allocation<Data>).free();
                }
            }
        }
    }

    pub unsafe fn manage<T: Trace + ?Sized>(self: Pin<&Self>, ptr: GcPtr<T>) {
        // TODO I should not need a dynamic check here but I am making mistakes
        if ptr.is_unmanaged() {
            self.objects().insert(ptr.erased_pinned());
        }
        ptr.data().manage();
    }

    pub fn push_root<T: Trace + ?Sized>(self: Pin<&Self>, root: GcPtr<T>) {
        let root: NonNull<Allocation<Data>> = root.erased();
        debug!("ENROOTING root at:          {:x}", root.as_ptr() as usize);
        self.roots.borrow_mut().push(root);
    }

    pub fn pop_root(self: Pin<&Self>) {
        if let Some(root) = self.roots.borrow_mut().pop() {
            debug!(" DROPPING root at:           {:x}", root.as_ptr() as usize);
        }
    }

    pub fn roots(&self) -> Ref<'_, [NonNull<Allocation<Data>>]> {
        Ref::map(self.roots.borrow(), |v| &v[..])
    }

    pub fn objects<'a>(self: Pin<&'a Self>) -> Pin<&'a List<Allocation<Data>>> {
        unsafe { Pin::map_unchecked(self, |this| &this.objects) }
    }
}
