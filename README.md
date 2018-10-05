# shifgrethor

## What is `shifgrethor`?

shifgrethor is an experiment. The goal is to define an API for precise, tracing
garbage collection in Rust which upholds all of Rust's safety guarantees. A
user using the API defined in this library will not be at risk for any of the
kinds of memory errors that Rust can prevent.

I believe the API presented in shifgrethor is genuinely safe & represents a
significant advancement on our understanding of how garbage collection could be
encoded in Rust.

## What kind of access does `shifgrethor` provide to data?

Some previous garbage collector APIs resolve some of the safety issues with
garbage collection by only allowing you to copy data out of them, rather than
allowing you to hold references directly into the managed memory. This is very
convenient for copying collectors as they don't have to implement pinning.

shifgrethor is not like those APIs. With shifgrethor, you can have direct
references into the managed heap. In fact, you can have arbitrary references
between the stack, the managed heap, and the unmanaged heap:

- Garbage collected objects can own data allocated in the unmanaged heap, and
  that data will be dropped when those objects are collected.
- Garbage collected objects can own references into the stack, and you are
  guaranteed not to be able to read from those references after they have gone
  out of scope in safe code.
- You can store pointers to garbage collected objects in the heap or on the
  stack.

The transitive combination of all of these is true: for example, a GC'd object
can own a heap-allocated vector of references to objects on the stack which
themselves have GC'd objects inside them.

Note that, like all garbage collection in Rust (e.g. `Rc`), shifgrethor only
provides immutable access to data it is managing. See the section on interior
mutability later.

## What kind of garbage collector is `shifgrethor`?

shifgrethor provides a garbage collector, but that is not what is interesting
about shifgrethor. The garbage collector here is a mark-and-sweep of the
simplest and least optimized possible variety. However, the API which makes it
safe could apply to much more performant garbage collectors, specifically with
these properties:

- This is an API for [tracing garbage collectors][tracing], not for other
  garbage collection techniques like reference counting.
- This is an API for [precise][precise] tracing collectors, not a conservative
  collector like the Boehme GC.
- The API could be trivially adapted to support concurrent GCs, though the
  current implementation is not thread safe.
- The API *can* support moving collectors as long as they implement a pinning
  mechanism. A moving collector which does not support pinning is incompatible
  with shifgrethor's API goals.

## What is the state of the project?

Code has been written, sometimes frantically. A few basic smoke tests of things
that should work working correctly has been done. No attempts at proofs have
been made. It likely has glaring bugs. It might seg fault, ruin your laundry,
halt and catch fire, etc.

**You should not use this for anything that you depend on (e.g. "in
production")!** But if you want to play around with it for fun, by all means.

## What is `shifgrethor` going to be used for?

No idea! This is currently a research project.

## Why is it called `shifgrethor`?

I created a new project called shifgrethor just to mess around it, the word
wasn't really meant to mean anything. Then that scratch project turned into
this, so the project is still called shifgrethor.

"Shifgrethor" is a word from *The Lefthand of Darkness* by Ursula K. Le Guin.

## How does `shifgrethor` work?

In brief, a precise tracing garbage collector like shifgrethor is designed for
works like this:

- All of the references from the unmanaged portion of memory (stack and heap,
  in our context) into the managed portion of memory are tracked. These are
  called *"roots."*
- From those roots, the collector *"traces"* through the graph of objects to
  find all of the objects that can still be accessed from those roots (and
  therefore, the objects which are still "alive.")

Our API needs to properly account for both rooting objects and tracing through
them to transitively rooted objects.

### Rooting

Given our limitations (i.e. no language support & the existence of a dynamic,
unmanaged heap), it is necessary that we track our roots through some sort of
intrusive collection. As a result, our roots cannot be moved around.

Fortunately, we have recently made a lot of progress on supporting intrusive
data structures in Rust, thanks to the pinning API. The rooting layer sits on
top of an underlying pinning API, which forms a doubly linked list of all of
your roots.

There are two kinds of roots:

- **Stack roots:** these roots are just an object in the stack. As a result,
  they are tied to the lifetime of that particular stackframe.
- **Heap roots:** these roots own a heap allocated pointer to a root in the
  heap. As a result, they are not tied to any particular lifetime, and you can
  move the root around, but they cost a single heap allocation per root.

Here is the API for each of them:

```rust
/*****************
 * Stack rooting *
 *****************/

// The root is declared with this macro:
letroot!(root);

// Create a new Gc'd object using that root
let x: Gc<i32> = root.gc(0);

/*****************
 * Heap rooting  *
 *****************/

let root: HeapRoot<i32> = HeapRoot::new(0);

// Unlike the stack root, which is consumed by this, this just borrows from
// the heap root.
let x: Gc<i32> = root.gc();
```

You can also reroot objects from one root to another, for example:

```rust
fn foo<'root>(root: StackRoot<'root>) -> Gc<'root, i32> {
    letroot!(root);

    // x only is rooted for the lifetime of this function call
    let x = root.gc(0);
    // ...

    // But you can reroot it later 
    root.reroot(x)
}
```

### Tracing

Its not enough to be able to root objects in the Gc, you also need to be able
to trace from the root to other objects *transitively*. For example, you might
want a struct, stored in the Gc, with fields pointing to other objects which
are also being garbage collected.

The problem that emerges is ensuring that you can only access transitively
rooted objects when you know they are actually being traced from a rooted
object. A few components enable us to solve this:

- First, to put a type in the garbage collector it must implement a trait which
  defines how to trace through it.
- Second, instead of only having a `Gc` type, we have a second type: `GcStore`.
- Using derived accessors, we can guarantee a safe API; let me explain:

The `Gc` type implements `Deref` and `Copy`, it functionally acts like a normal
reference, except that you can extend its lifetime by rerooting it. It does not
expose a safe API for constructing it: the only constructor is an unsafe
`Gc::rooted` constructor: to safely call this constructor, you must prove that
this will be rooted for the lifetime `'root`.

The `GcStore` type is more like a `Box`, except that it does not implement
`Deref`. You can safely construct a `GcStore`, which will have `Box` semantics
until it is rooted - that is, if you drop a `GcStore` without having rooted it
first, it will deallocate what you have put into it.

Finally, as a part of the same derive which implements the traits necessary to
garbage collect your type, you can implement an accessor to transform your
`GcStore` fields into `Gc` fields. For example:

```rust
#[derive(GC)]
struct Foo<'root> {
    #[gc] bar: GcStore<'root, Bar>,
}
```

This code gives generates this method on Foo:

```rust
fn bar(self: Gc<'root, Foo<'_>>) -> Gc<'root, Bar>
```

Because the derive also guarantees that this field is traced properly, if you
have a `Gc<Foo>`, it is safe to construct a `Gc<Bar>` from it.

This behavior is also implemented for several container types. For example, you
can transform a `Vec<GcStore<_>>` to a `Vec<Gc>` in the same way:

```rust
#[derive(GC)]
struct Foo<'root> {
    #[gc] vec: Vec<GcStore<'root, Bar>>,
}

// Generates:
fn vec<'root>(self: Gc<'root, Self>) -> Vec<Gc<'root, Bar>>;
```

### Destructors

Destructors present a troubling problem for garbage collectors. Destructors are
safe because we can guarantee that they are run when the struct is dropped, but
something garbage collected will not actually be dropped (and the destructor
run) until much later. This can cause two problems:

* If the destructor accesses other Gc'd data, that data might have been freed
  earlier by the collector.
* If the destructor accesses data on the stack, that data might have been freed
  when the stack was popped before the collector ran.

As a result, the GC does not run destructors on its objects. Instead, it runs a
finalizer just before collecting each object. You can define what happens in
the finalizer by implementing the `Finalize` trait for your type and adding a
`#[gc(finalize)]` attribute to your struct:

```rust
#[derive(GC)]
#[gc(finalize)]
struct Foo;

impl shifgrethor::Finalize for Foo {
    fn finalize(&mut self) {
        println!("Collecting a Foo");
    }
}
```

Because `Finalize` does not give you a `Gc` pointer to your type, you cannot
access other `Gc` pointers (in other words, you cannot "prove rootedness"
because you are no longer rooted in the finalizer.) However, this is
insufficient for preventing you from accessing other non-owned data, like stack
references.

As a result, if your type contains any lifetimes other than `'root`, attempting
to implement a finalizer like this will fail. Instead, you will need to
implement an unsafe finalizer:

```rust
#[derive(GC)]
#[gc(unsafe_finalize)]
struct Foo<'a>(&'a i32);

unsafe impl shifgrethor::UnsafeFinalize for Foo {
    fn finalize(&mut self) {
        println!("Collecting a Foo");
    }
}
```

You must audit these finalizers and guarantee that your finalizer never reads
from the any of the borrowed references inside of it, otherwise your code is
not safe and contains undefined behavior.

### Interior mutability

The final problem is interior mutability. Currently, shifgrethor does not
support any interior mutability at all: once something is being garbage
collected, you can no longer mutate it. Solving this problem is now the focus
of the research in shifgrethor.

I have an adequate solution for interior mutability for data which does not
contain any pointers to other GC'd objects which I intend to implement soon.
For interior mutability of things which contain Gc'd objects, I am still
investigating some possible alternatives.

[tracing]: https://en.wikipedia.org/wiki/Tracing_garbage_collection
[precise]: https://en.wikipedia.org/wiki/Tracing_garbage_collection#Precise_vs._conservative_and_internal_pointers
