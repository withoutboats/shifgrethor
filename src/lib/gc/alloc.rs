use std::cell::Cell;
use std::mem;
use std::ptr::NonNull;

use log::*;

use crate::list::List;
use crate::trace::Trace;

extern {
    pub type Data;
    type Vtable;
}

pub struct Allocation<T: ?Sized> {
    header: Header,
    pub(crate) data: T,
}

struct Header {
    list: List<Allocation<Data>>,
    vtable: *mut Vtable,
    marked: Cell<bool>,
}

impl<T: Trace> Allocation<T> {
    pub fn new(data: T) -> NonNull<Allocation<T>> {
        let vtable = extract_vtable(&data);

        let allocation = Box::new(Allocation {
            header: Header {
                list: List::default(),
                vtable: vtable,
                marked: Cell::new(false),
            },
            data,
        });
        unsafe {
            NonNull::new_unchecked(Box::into_raw(allocation))
        }
    }
}

impl Allocation<Data> {
    pub unsafe fn free(self: *mut Allocation<Data>) {
        (&mut *self).dyn_data_mut().finalize();
        drop(Box::from_raw(self))
    }
}

impl<T: ?Sized> Allocation<T> {
    pub unsafe fn mark(&self) {
        debug!("MARKING object at:          {:x}", self.erased() as *const _ as usize);
        if !self.header.marked.replace(true) {
            self.dyn_data().mark()
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn marked(&self) -> bool {
        self.header.marked.replace(false)
    }

    pub fn is_unmanaged(&self) -> bool {
        self.header.list.is_head()
    }

    fn dyn_data(&self) -> &dyn Trace {
        unsafe {
            let object = Object {
                data: self.erased().data() as *const Data,
                vtable: self.header.vtable,
            };
            mem::transmute::<Object, &dyn Trace>(object)
        }
    }

    fn dyn_data_mut(&mut self) -> &mut dyn Trace {
        unsafe {
            let object = Object {
                data: self.erased().data() as *const Data,
                vtable: self.header.vtable,
            };
            mem::transmute::<Object, &mut dyn Trace>(object)
        }
    }

    fn erased(&self) -> &Allocation<Data> {
        unsafe {
            &*(self as *const Allocation<T> as *const Allocation<Data>)
        }
    }
}

impl AsRef<List<Allocation<Data>>> for Allocation<Data> {
    fn as_ref(&self) -> &List<Allocation<Data>> {
        &self.header.list
    }
}

#[repr(C)]
struct Object {
    data: *const Data,
    vtable: *mut Vtable,
}

fn extract_vtable<T: Trace>(data: &T) -> *mut Vtable {
    unsafe {
        let obj = data as &Trace;
        mem::transmute::<&dyn Trace, Object>(obj).vtable
    }
}
