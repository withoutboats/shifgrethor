use std::ptr::NonNull;

use log::*;

use crate::alloc::{Allocation, Data};
use crate::list::List;
use crate::trace::Trace;
use crate::gc_ptr::GcPtr;

pub struct Root {
    list: List<Root>,
    alloc: NonNull<Allocation<Data>>,
}

impl Root {
    /// A root (not yet enrooted).
    ///
    /// The pointer you pass must not be dangling.
    pub unsafe fn new<T: Trace + ?Sized>(gc_ptr: GcPtr<T>) -> Root {
        Root {
            list: List::default(),
            alloc: gc_ptr.erased(),
        }
    }

    pub(crate) fn mark(&self) {
        unsafe {
            self.alloc.as_ref().mark();
        }

    }
}

impl AsRef<List<Root>> for Root {
    fn as_ref(&self) -> &List<Root> {
        &self.list
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        debug!(" DROPPING root at:           {:x}", self as *const _ as usize);
    }
}
