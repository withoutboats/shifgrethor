use crate::gc_ptr::GcPtr;
use crate::trace::Trace;

pub struct Root {
    idx: usize,
}

impl Root {
    pub fn new() -> Root {
        Root { idx: super::new_root() }
    }

    pub unsafe fn enroot<T: Trace + ?Sized>(&self, gc_ptr: GcPtr<T>) {
        super::set_root(self.idx, gc_ptr)
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        super::pop_root(self.idx);
    }
}
