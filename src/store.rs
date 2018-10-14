use crate::{Gc, GcStore};

pub unsafe trait Store<'root> {
    type Accessor: 'root;
    unsafe fn rooted(this: &'root Self) -> Self::Accessor;
}

unsafe impl<'root, 'r, T: ?Sized + 'root> Store<'root> for GcStore<'r, T> {
    type Accessor = Gc<'root, T>;
    unsafe fn rooted(this: &'root Self) -> Self::Accessor {
        Gc::rooted(GcStore::raw(this))
    }
}

macro_rules! transmute_store {
    ($(for<$($T:ident),*> $from:ty => $to:ty;)*) => {$(
        unsafe impl<'root, 'r, $($T: ?Sized + 'root,)*> Store<'root> for $from {
            type Accessor = &'root $to;
            unsafe fn rooted(this: &'root $from) -> &'root $to {
                std::mem::transmute::<&'root $from, &'root $to>(this)
            }
        }
    )*}
}


use std::collections::*;
use pin_cell::PinCell;

transmute_store! {
    for<T> Box<GcStore<'r, T>> => Box<Gc<'root, T>>;
    for<T> Option<GcStore<'r, T>> => Option<Gc<'root, T>>;
    for<T> [GcStore<'r, T>] => [Gc<'root, T>];
    for<T> Vec<GcStore<'r, T>> => Vec<Gc<'root, T>>;
    for<T> VecDeque<GcStore<'r, T>> => VecDeque<Gc<'root, T>>;
    for<T> HashSet<GcStore<'r, T>> => HashSet<Gc<'root, T>>;
    for<T> BTreeSet<GcStore<'r, T>> => BTreeSet<Gc<'root, T>>;
    for<T> BinaryHeap<GcStore<'r, T>> => BinaryHeap<Gc<'root, T>>;
    for<T> PinCell<GcStore<'r, T>> => PinCell<Gc<'root, T>>;
}
