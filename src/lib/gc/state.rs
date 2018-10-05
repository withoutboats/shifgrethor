use std::pin::Pin;

use log::*;

use crate::alloc::{Allocation, Data};
use crate::gc_ptr::GcPtr;
use crate::list::List;
use crate::root::Root;
use crate::trace::Trace;

#[derive(Default)]
pub struct GcState {
    objects: List<Allocation<Data>>,
    roots: List<Root>,
}

impl GcState {
    pub fn collect(self: Pin<&Self>) {
        for root in self.roots() {
            debug!("TRACING from root at:       {:x}", &*root as *const _ as usize);
            root.mark();
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

    pub fn enroot(self: Pin<&Self>, root: Pin<&Root>) {
        debug!("ENROOTING root at:          {:x}", &*root as *const _ as usize);
        self.roots().insert(root);
    }

    pub fn objects<'a>(self: Pin<&'a Self>) -> Pin<&'a List<Allocation<Data>>> {
        unsafe { Pin::map_unchecked(self, |this| &this.objects) }
    }

    pub fn roots<'a>(self: Pin<&'a Self>) -> Pin<&'a List<Root>> {
        unsafe { Pin::map_unchecked(self, |this| &this.roots) }
    }
}
