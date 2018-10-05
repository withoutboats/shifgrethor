use std::pin::Pin;

use gc::{GcPtr, Trace, Root};

use crate::Gc;
use crate::root::Reroot;

pub struct StackRoot<'root> {
    root: Pin<&'root mut Option<Root>>,
}

impl<'root> StackRoot<'root> {
    #[doc(hidden)]
    pub fn new(root: Pin<&'root mut Option<Root>>) -> StackRoot<'root> {
        StackRoot { root }
    }

    pub fn gc<T>(self, data: T) -> Gc<'root, T::Rerooted> where
        T: Reroot<'root> + Trace,
        T::Rerooted: Trace,
    {
        unsafe {
            self.make(gc::alloc_unmanaged(data))
        }
    }

    pub fn reroot<T>(self, gc: Gc<'_, T>) -> Gc<'root, T::Rerooted> where
        T: Reroot<'root> + ?Sized,
        T::Rerooted: Trace,
    {
        unsafe {
            self.make(Gc::raw(gc))
        }
    }

    pub(crate) unsafe fn make<T>(mut self, ptr: GcPtr<T>) -> Gc<'root, T::Rerooted> where
        T: Reroot<'root> + ?Sized,
        T::Rerooted: Trace,
    {
        let ptr = super::reroot(ptr);
        self.emplace(ptr);
        Gc::rooted(ptr)
    }

    unsafe fn emplace<T: Trace + ?Sized>(&mut self, ptr: GcPtr<T>) {
        let new_root = Root::new(ptr);
        let pin: Pin<&mut Root> = Pin::map_unchecked_mut(Pin::as_mut(&mut self.root), |root| {
            *root = Some(new_root);
            root.as_mut().unwrap()
        });
        gc::enroot(Pin::as_ref(&pin))
    }
}

#[doc(hidden)]
pub unsafe fn pin_root(root: &mut Option<Root>) -> Pin<&mut Option<Root>> {
    Pin::new_unchecked(root)
}

#[macro_export]
macro_rules! letroot {
    ($($root:ident)*) => {$(
        // Ensure the root is owned
        let mut $root = None;

        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $root = unsafe {
            $crate::StackRoot::new($crate::pin_root(&mut $root))
        };
    )*}
}
