// But it was from the difference between us, not from the affinities and
// likenesses, but from the difference, that that love came: and it was itself
// the bridge, the only bridge, across what divided us.
//      - Ursula K. Le Guin
#![feature(pin)]

mod gc;
mod gc_store;
mod no_trace;
mod root;
mod store;

#[cfg(test)]
mod tests;

pub use ::gc::{Trace, NullTrace, collect};
pub use derive::*;

pub mod raw {
    pub use gc::{GcPtr, alloc, alloc_unmanaged, manage, enroot};
    pub use gc::{count_managed_objects, count_roots};
}

pub use self::gc::*;
pub use self::gc_store::*;
pub use self::no_trace::*;
pub use self::root::*;
pub use self::store::*;

pub trait Finalize {
    fn finalize(&mut self);
}

pub unsafe trait UnsafeFinalize {
    fn finalize(&mut self);
}

impl<T: UnsafeFinalize + ?Sized> Finalize for T {
    fn finalize(&mut self) {
        UnsafeFinalize::finalize(self)
    }
}

pub trait GC<'root>: Reroot<'root> + Trace { }
