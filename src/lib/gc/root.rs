use crate::gc_ptr::GcPtr;
use crate::trace::Trace;

pub struct Root {
    pushed: bool,
}

impl Root {
    pub fn new() -> Root {
        Root { pushed: false }
    }

    pub unsafe fn enroot<T: Trace + ?Sized>(&mut self, gc_ptr: GcPtr<T>) {
        super::push_root(gc_ptr);
        self.pushed = true;
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        if self.pushed {
            super::pop_root();
        }
    }
}
