use std::cell;
use std::mem;

use gc::{GcPtr, Trace, NullTrace};

use crate::{Gc, GcStore};

pub unsafe trait Reroot<'root> {
    type Rerooted: ?Sized + 'root;
}

pub unsafe fn reroot<'root, T>(data: GcPtr<T>) -> GcPtr<T::Rerooted> where
    T: Reroot<'root> + ?Sized,
    T::Rerooted: Trace,
{
    let ptr: GcPtr<T::Rerooted> = mem::transmute_copy(&data);
    gc::manage::<T::Rerooted>(ptr);
    ptr
}

unsafe impl<'root, T: Reroot<'root> + ?Sized> Reroot<'root> for GcPtr<T> {
    type Rerooted = GcPtr<T::Rerooted>;
}

unsafe impl<'root, 'r2, T: Reroot<'root> + ?Sized> Reroot<'root> for Gc<'r2, T> {
    type Rerooted = Gc<'root, T::Rerooted>;
}

unsafe impl<'root, 'r2, T: Reroot<'root> + ?Sized> Reroot<'root> for GcStore<'r2, T> {
    type Rerooted = GcStore<'root, T::Rerooted>;
}

unsafe impl<'root, T: Reroot<'root> + ?Sized> Reroot<'root> for pin_cell::PinCell<T> {
    type Rerooted = pin_cell::PinCell<T::Rerooted>;
}

unsafe impl<'root, T: NullTrace + Reroot<'root> + ?Sized> Reroot<'root> for cell::Cell<T> {
    type Rerooted = cell::Cell<T::Rerooted>;
}

unsafe impl<'root, T: NullTrace + Reroot<'root> + ?Sized> Reroot<'root> for cell::RefCell<T> {
    type Rerooted = cell::RefCell<T::Rerooted>;
}

macro_rules! reroot_simple {
    ($($t:ty)*) => {$(unsafe impl<'root> Reroot<'root> for $t {
        type Rerooted = $t;
    })*}
}

reroot_simple!(
    i8  i16 i32 i64 isize
    u8  u16 u32 u64 usize
    f32     f64
    char    bool
    str     String
    std::fs::File
    std::fs::FileType
    std::fs::Metadata
    std::fs::OpenOptions
    std::io::BufRead
    std::io::Read
    std::io::Write
    std::io::Stdin
    std::io::Stdout
    std::io::Stderr
    std::io::Error
    std::net::TcpStream
    std::net::TcpListener
    std::net::UdpSocket
    std::net::Ipv4Addr
    std::net::Ipv6Addr
    std::net::SocketAddrV4
    std::net::SocketAddrV6
    std::path::Path
    std::path::PathBuf
    std::process::Command
    std::process::Child
    std::process::ChildStdout
    std::process::ChildStdin
    std::process::ChildStderr
    std::process::Output
    std::process::ExitStatus
    std::process::Stdio
    std::sync::Barrier
    std::sync::Condvar
    std::sync::Once
);

macro_rules! reroot_arrays {
    ($($N:expr),*)  => {$(
        unsafe impl<'root, T: Reroot<'root>> Reroot<'root> for [T; $N]
            where T::Rerooted: Sized,
        {
            type Rerooted = [T::Rerooted; $N];
        }
    )*};
}

reroot_arrays! {
    0o00, 0o01, 0o02, 0o03, 0o04, 0o05, 0o06, 0o07,
    0o10, 0o11, 0o12, 0o13, 0o14, 0o15, 0o16, 0o17,
    0o20, 0o21, 0o22, 0o23, 0o24, 0o25, 0o26, 0o27,
    0o30, 0o31, 0o32, 0o33, 0o34, 0o35, 0o36, 0o37
}


macro_rules! reroot_tuples {
    ($(($($T:ident),*))*) => {$(
        unsafe impl<'root, $($T: Reroot<'root>,)*> Reroot<'root> for ($($T,)*) where
            $($T::Rerooted: Sized,)*
        {
            type Rerooted = ($($T::Rerooted,)*);
        }
    )*};
}

reroot_tuples! {
    ()
    (A)
    (A, B)
    (A, B, C)
    (A, B, C, D)
    (A, B, C, D, E)
    (A, B, C, D, E, F)
    (A, B, C, D, E, F, G)
    (A, B, C, D, E, F, G, H)
    (A, B, C, D, E, F, G, H, I)
    (A, B, C, D, E, F, G, H, I, J)
    (A, B, C, D, E, F, G, H, I, J, K)
    (A, B, C, D, E, F, G, H, I, J, K, L)
}

use std::collections::*;
use std::rc::Rc;
use std::sync::Arc;

macro_rules! reroot_generic {
    ($($Type:ident<$($T:ident),*>),*) => {$(
        unsafe impl<'root, $($T,)*> Reroot<'root> for $Type<$($T,)*> where
            $($T: ?Sized + Reroot<'root>,)*
        {
            type Rerooted = $Type<$($T::Rerooted,)*>;
        }
    )*}
}

macro_rules! reroot_generic_sized {
    ($($Type:ident<$($T:ident),*>),*) => {$(
        unsafe impl<'root, $($T,)*> Reroot<'root> for $Type<$($T,)*> where
            $($T: Reroot<'root>,)*
            $($T::Rerooted: Sized,)*
        {
            type Rerooted = $Type<$($T::Rerooted,)*>;
        }
    )*}
}

reroot_generic! {
    Box<T>, Rc<T>, Arc<T>
}

reroot_generic_sized! {
    Option<T>, Result<T, E>,
    Vec<T>, VecDeque<T>, LinkedList<T>, BinaryHeap<T>,
    HashMap<K, V, H>, HashSet<T, H>, BTreeMap<K, V>, BTreeSet<T>
}

// TODO more impls
