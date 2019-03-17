//! Types that pin data to its location in memory.
//!
//! It is sometimes useful to have objects that are guaranteed to not move,
//! in the sense that their placement in memory does not change, and can thus be relied upon.
//! A prime example of such a scenario would be building self-referential structs,
//! since moving an object with pointers to itself will invalidate them,
//! which could cause undefined behavior.
//!
//! A [`Pin<P>`] ensures that the pointee of any pointer type `P` has a stable location in memory,
//! meaning it cannot be moved elsewhere and its memory cannot be deallocated
//! until it gets dropped. We say that the pointee is "pinned".
//!
//! By default, all types in Rust are movable. Rust allows passing all types by-value,
//! and common smart-pointer types such as `Box<T>` and `&mut T` allow replacing and
//! moving the values they contain: you can move out of a `Box<T>`, or you can use [`mem::swap`].
//! [`Pin<P>`] wraps a pointer type `P`, so `Pin<Box<T>>` functions much like a regular `Box<T>`:
//! when a `Pin<Box<T>>` gets dropped, so do its contents, and the memory gets deallocated.
//! Similarily, `Pin<&mut T>` is a lot like `&mut T`. However, [`Pin<P>`] does not let clients
//! actually obtain a `Box<T>` or `&mut T` to pinned data, which implies that you cannot use
//! operations such as [`mem::swap`]:
//! ```
//! use std::pin::Pin;
//! fn swap_pins<T>(x: Pin<&mut T>, y: Pin<&mut T>) {
//!     // `mem::swap` needs `&mut T`, but we cannot get it.
//!     // We are stuck, we cannot swap the contents of these references.
//!     // We could use `Pin::get_unchecked_mut`, but that is unsafe for a reason:
//!     // we are not allowed to use it for moving things out of the `Pin`.
//! }
//! ```
//!
//! It is worth reiterating that [`Pin<P>`] does *not* change the fact that a Rust compiler
//! considers all types movable. [`mem::swap`] remains callable for any `T`. Instead, `Pin<P>`
//! prevents certain *values* (pointed to by pointers wrapped in `Pin<P>`) from being
//! moved by making it impossible to call methods that require `&mut T` on them
//! (like [`mem::swap`]).
//!
//! [`Pin<P>`] can be used to wrap any pointer type `P`, and as such it interacts with
//! [`Deref`] and [`DerefMut`]. A `Pin<P>` where `P: Deref` should be considered
//! as a "`P`-style pointer" to a pinned `P::Target` -- so, a `Pin<Box<T>>` is
//! an owned pointer to a pinned `T`, and a `Pin<Rc<T>>` is a reference-counted
//! pointer to a pinned `T`.
//! For correctness, [`Pin<P>`] relies on the [`Deref`] and [`DerefMut`] implementations
//! to not move out of their `self` parameter, and to only ever return a pointer
//! to pinned data when they are called on a pinned pointer.
//!
//! # `Unpin`
//!
//! However, these restrictions are usually not necessary. Many types are always freely
//! movable, even when pinned, because they do not rely on having a stable address.
//! This includes all the basic types (like `bool`, `i32`, references)
//! as well as types consisting solely of these types.
//! Types that do not care about pinning implement the [`Unpin`] auto-trait, which
//! cancels the effect of [`Pin<P>`]. For `T: Unpin`, `Pin<Box<T>>` and `Box<T>` function
//! identically, as do `Pin<&mut T>` and `&mut T`.
//!
//! Note that pinning and `Unpin` only affect the pointed-to type `P::Target`, not the pointer
//! type `P` itself that got wrapped in `Pin<P>`. For example, whether or not `Box<T>` is
//! `Unpin` has no effect on the behavior of `Pin<Box<T>>` (here, `T` is the
//! pointed-to type).
//!
//! # Example: self-referential struct
//!
//! ```rust
//! use std::pin::Pin;
//! use std::marker::PhantomPinned;
//! use std::ptr::NonNull;
//!
//! // This is a self-referential struct since the slice field points to the data field.
//! // We cannot inform the compiler about that with a normal reference,
//! // since this pattern cannot be described with the usual borrowing rules.
//! // Instead we use a raw pointer, though one which is known to not be null,
//! // since we know it's pointing at the string.
//! struct Unmovable {
//!     data: String,
//!     slice: NonNull<String>,
//!     _pin: PhantomPinned,
//! }
//!
//! impl Unmovable {
//!     // To ensure the data doesn't move when the function returns,
//!     // we place it in the heap where it will stay for the lifetime of the object,
//!     // and the only way to access it would be through a pointer to it.
//!     fn new(data: String) -> Pin<Box<Self>> {
//!         let res = Unmovable {
//!             data,
//!             // we only create the pointer once the data is in place
//!             // otherwise it will have already moved before we even started
//!             slice: NonNull::dangling(),
//!             _pin: PhantomPinned,
//!         };
//!         let mut boxed = Box::pin(res);
//!
//!         let slice = NonNull::from(&boxed.data);
//!         // we know this is safe because modifying a field doesn't move the whole struct
//!         unsafe {
//!             let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
//!             Pin::get_unchecked_mut(mut_ref).slice = slice;
//!         }
//!         boxed
//!     }
//! }
//!
//! let unmoved = Unmovable::new("hello".to_string());
//! // The pointer should point to the correct location,
//! // so long as the struct hasn't moved.
//! // Meanwhile, we are free to move the pointer around.
//! # #[allow(unused_mut)]
//! let mut still_unmoved = unmoved;
//! assert_eq!(still_unmoved.slice, NonNull::from(&still_unmoved.data));
//!
//! // Since our type doesn't implement Unpin, this will fail to compile:
//! // let new_unmoved = Unmovable::new("world".to_string());
//! // std::mem::swap(&mut *still_unmoved, &mut *new_unmoved);
//! ```
//!
//! # Example: intrusive doubly-linked list
//!
//! In an intrusive doubly-linked list, the collection does not actually allocate
//! the memory for the elements itself. Allocation is controlled by the clients,
//! and elements can live on a stack frame that lives shorter than the collection does.
//!
//! To make this work, every element has pointers to its predecessor and successor in
//! the list. Elements can only be added when they are pinned, because moving the elements
//! around would invalidate the pointers. Moreover, the `Drop` implementation of a linked
//! list element will patch the pointers of its predecessor and successor to remove itself
//! from the list.
//!
//! Crucially, we have to be able to rely on `drop` being called. If an element
//! could be deallocated or otherwise invalidated without calling `drop`, the pointers into it
//! from its neighbouring elements would become invalid, which would break the data structure.
//!
//! Therefore, pinning also comes with a `drop`-related guarantee.
//!
//! # `Drop` guarantee
//!
//! The purpose of pinning is to be able to rely on the placement of some data in memory.
//! To make this work, not just moving the data is restricted; deallocating, repurposing, or
//! otherwise invalidating the memory used to store the data is restricted, too.
//! Concretely, for pinned data you have to maintain the invariant
//! that *its memory will not get invalidated from the moment it gets pinned until
//! when `drop` is called*. Memory can be invalidated by deallocation, but also by
//! replacing a [`Some(v)`] by [`None`], or calling [`Vec::set_len`] to "kill" some elements
//! off of a vector.
//!
//! This is exactly the kind of guarantee that the intrusive linked list from the previous
//! section needs to function correctly.
//!
//! Notice that this guarantee does *not* mean that memory does not leak! It is still
//! completely okay not to ever call `drop` on a pinned element (e.g., you can still
//! call [`mem::forget`] on a `Pin<Box<T>>`). In the example of the doubly-linked
//! list, that element would just stay in the list. However you may not free or reuse the storage
//! *without calling `drop`*.
//!
//! # `Drop` implementation
//!
//! If your type uses pinning (such as the two examples above), you have to be careful
//! when implementing `Drop`. The `drop` function takes `&mut self`, but this
//! is called *even if your type was previously pinned*! It is as if the
//! compiler automatically called `get_unchecked_mut`.
//!
//! This can never cause a problem in safe code because implementing a type that relies on pinning
//! requires unsafe code, but be aware that deciding to make use of pinning
//! in your type (for example by implementing some operation on `Pin<&[mut] Self>`)
//! has consequences for your `Drop` implementation as well: if an element
//! of your type could have been pinned, you must treat Drop as implicitly taking
//! `Pin<&mut Self>`.
//!
//! In particular, if your type is `#[repr(packed)]`, the compiler will automatically
//! move fields around to be able to drop them. As a consequence, you cannot use
//! pinning with a `#[repr(packed)]` type.
//!
//! # Projections and Structural Pinning
//!
//! One interesting question arises when considering the interaction of pinning and
//! the fields of a struct. When can a struct have a "pinning projection", i.e.,
//! an operation with type `fn(Pin<&[mut] Struct>) -> Pin<&[mut] Field>`?
//! In a similar vein, when can a generic wrapper type (such as `Vec<T>`, `Box<T>`, or `RefCell<T>`)
//! have an operation with type `fn(Pin<&[mut] Wrapper<T>>) -> Pin<&[mut] T>`?
//!
//! Having a pinning projection for some field means that pinning is "structural":
//! when the wrapper is pinned, the field must be considered pinned, too.
//! After all, the pinning projection lets us get a `Pin<&[mut] Field>`.
//!
//! However, structural pinning comes with a few extra requirements, so not all
//! wrappers can be structural and hence not all wrappers can offer pinning projections:
//!
//! 1.  The wrapper must only be [`Unpin`] if all the structural fields are
//!     `Unpin`. This is the default, but `Unpin` is a safe trait, so as the author of
//!     the wrapper it is your responsibility *not* to add something like
//!     `impl<T> Unpin for Wrapper<T>`. (Notice that adding a projection operation
//!     requires unsafe code, so the fact that `Unpin` is a safe trait  does not break
//!     the principle that you only have to worry about any of this if you use `unsafe`.)
//! 2.  The destructor of the wrapper must not move structural fields out of its argument. This
//!     is the exact point that was raised in the [previous section][drop-impl]: `drop` takes
//!     `&mut self`, but the wrapper (and hence its fields) might have been pinned before.
//!     You have to guarantee that you do not move a field inside your `Drop` implementation.
//!     In particular, as explained previously, this means that your wrapper type must *not*
//!     be `#[repr(packed)]`.
//! 3.  You must make sure that you uphold the [`Drop` guarantee][drop-guarantee]:
//!     once your wrapper is pinned, the memory that contains the
//!     content is not overwritten or deallocated without calling the content's destructors.
//!     This can be tricky, as witnessed by `VecDeque<T>`: the destructor of `VecDeque<T>` can fail
//!     to call `drop` on all elements if one of the destructors panics. This violates the
//!     `Drop` guarantee, because it can lead to elements being deallocated without
//!     their destructor being called. (`VecDeque` has no pinning projections, so this
//!     does not cause unsoundness.)
//! 4.  You must not offer any other operations that could lead to data being moved out of
//!     the fields when your type is pinned. For example, if the wrapper contains an
//!     `Option<T>` and there is a `take`-like operation with type
//!     `fn(Pin<&mut Wrapper<T>>) -> Option<T>`,
//!     that operation can be used to move a `T` out of a pinned `Wrapper<T>` -- which means
//!     pinning cannot be structural.
//!
//!     For a more complex example of moving data out of a pinned type, imagine if `RefCell<T>`
//!     had a method `fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T>`.
//!     Then we could do the following:
//!     ```compile_fail
//!     fn exploit_ref_cell<T>(rc: Pin<&mut RefCell<T>>) {
//!         { let p = rc.as_mut().get_pin_mut(); } // Here we get pinned access to the `T`.
//!         let rc_shr: &RefCell<T> = rc.into_ref().get_ref();
//!         let b = rc_shr.borrow_mut();
//!         let content = &mut *b; // And here we have `&mut T` to the same data.
//!     }
//!     ```
//!     This is catastrophic, it means we can first pin the content of the `RefCell<T>`
//!     (using `RefCell::get_pin_mut`) and then move that content using the mutable
//!     reference we got later.
//!
//! For a type like `Vec<T>`, both possibilites (structural pinning or not) make sense,
//! and the choice is up to the author. A `Vec<T>` with structural pinning could
//! have `get_pin`/`get_pin_mut` projections. However, it could *not* allow calling
//! `pop` on a pinned `Vec<T>` because that would move the (structurally pinned) contents!
//! Nor could it allow `push`, which might reallocate and thus also move the contents.
//! A `Vec<T>` without structural pinning could `impl<T> Unpin for Vec<T>`, because the contents
//! are never pinned and the `Vec<T>` itself is fine with being moved as well.
//!
//! In the standard library, pointer types generally do not have structural pinning,
//! and thus they do not offer pinning projections. This is why `Box<T>: Unpin` holds for all `T`.
//! It makes sense to do this for pointer types, because moving the `Box<T>`
//! does not actually move the `T`: the `Box<T>` can be freely movable (aka `Unpin`) even if the `T`
//! is not. In fact, even `Pin<Box<T>>` and `Pin<&mut T>` are always `Unpin` themselves,
//! for the same reason: their contents (the `T`) are pinned, but the pointers themselves
//! can be moved without moving the pinned data. For both `Box<T>` and `Pin<Box<T>>`,
//! whether the content is pinned is entirely independent of whether the pointer is
//! pinned, meaning pinning is *not* structural.
//!
//! [`Pin<P>`]: struct.Pin.html
//! [`Unpin`]: ../../std/marker/trait.Unpin.html
//! [`Deref`]: ../../std/ops/trait.Deref.html
//! [`DerefMut`]: ../../std/ops/trait.DerefMut.html
//! [`mem::swap`]: ../../std/mem/fn.swap.html
//! [`mem::forget`]: ../../std/mem/fn.forget.html
//! [`Box<T>`]: ../../std/boxed/struct.Box.html
//! [`Vec::set_len`]: ../../std/vec/struct.Vec.html#method.set_len
//! [`None`]: ../../std/option/enum.Option.html#variant.None
//! [`Some(v)`]: ../../std/option/enum.Option.html#variant.Some
//! [drop-impl]: #drop-implementation
//! [drop-guarantee]: #drop-guarantee

#![stable(feature = "pin", since = "1.33.0")]

use fmt;
use marker::{Sized, Unpin};
use cmp::{self, PartialEq, PartialOrd};
use ops::{Deref, DerefMut, Receiver, CoerceUnsized, DispatchFromDyn};

/// A pinned pointer.
///
/// This is a wrapper around a kind of pointer which makes that pointer "pin" its
/// value in place, preventing the value referenced by that pointer from being moved
/// unless it implements [`Unpin`].
///
/// See the [`pin` module] documentation for further explanation on pinning.
///
/// [`Unpin`]: ../../std/marker/trait.Unpin.html
/// [`pin` module]: ../../std/pin/index.html
//
// Note: the derives below, and the explicit `PartialEq` and `PartialOrd`
// implementations, are allowed because they all only use `&P`, so they cannot move
// the value behind `pointer`.
#[stable(feature = "pin", since = "1.33.0")]
#[lang = "pin"]
#[fundamental]
#[repr(transparent)]
#[derive(Copy, Clone, Hash, Eq, Ord)]
pub struct Pin<P> {
    pointer: P,
}

#[stable(feature = "pin_partialeq_partialord_impl_applicability", since = "1.34.0")]
impl<P, Q> PartialEq<Pin<Q>> for Pin<P>
where
    P: PartialEq<Q>,
{
    fn eq(&self, other: &Pin<Q>) -> bool {
        self.pointer == other.pointer
    }

    fn ne(&self, other: &Pin<Q>) -> bool {
        self.pointer != other.pointer
    }
}

#[stable(feature = "pin_partialeq_partialord_impl_applicability", since = "1.34.0")]
impl<P, Q> PartialOrd<Pin<Q>> for Pin<P>
where
    P: PartialOrd<Q>,
{
    fn partial_cmp(&self, other: &Pin<Q>) -> Option<cmp::Ordering> {
        self.pointer.partial_cmp(&other.pointer)
    }

    fn lt(&self, other: &Pin<Q>) -> bool {
        self.pointer < other.pointer
    }

    fn le(&self, other: &Pin<Q>) -> bool {
        self.pointer <= other.pointer
    }

    fn gt(&self, other: &Pin<Q>) -> bool {
        self.pointer > other.pointer
    }

    fn ge(&self, other: &Pin<Q>) -> bool {
        self.pointer >= other.pointer
    }
}

impl<P: Deref> Pin<P>
where
    P::Target: Unpin,
{
    /// Construct a new `Pin<P>` around a pointer to some data of a type that
    /// implements [`Unpin`].
    ///
    /// Unlike `Pin::new_unchecked`, this method is safe because the pointer
    /// `P` dereferences to an [`Unpin`] type, which cancels the pinning guarantees.
    ///
    /// [`Unpin`]: ../../std/marker/trait.Unpin.html
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub fn new(pointer: P) -> Pin<P> {
        // Safety: the value pointed to is `Unpin`, and so has no requirements
        // around pinning.
        unsafe { Pin::new_unchecked(pointer) }
    }
}

impl<P: Deref> Pin<P> {
    /// Construct a new `Pin<P>` around a reference to some data of a type that
    /// may or may not implement `Unpin`.
    ///
    /// If `pointer` dereferences to an `Unpin` type, `Pin::new` should be used
    /// instead.
    ///
    /// # Safety
    ///
    /// This constructor is unsafe because we cannot guarantee that the data
    /// pointed to by `pointer` is pinned, meaning that the data will not be moved or
    /// its storage invalidated until it gets dropped. If the constructed `Pin<P>` does
    /// not guarantee that the data `P` points to is pinned, that is a violation of
    /// the API contract and may lead to undefined behavior in later (safe) operations.
    ///
    /// By using this method, you are making a promise about the `P::Deref` and
    /// `P::DerefMut` implementations, if they exist. Most importantly, they
    /// must not move out of their `self` arguments: `Pin::as_mut` and `Pin::as_ref`
    /// will call `DerefMut::deref_mut` and `Deref::deref` *on the pinned pointer*
    /// and expect these methods to uphold the pinning invariants.
    /// Moreover, by calling this method you promise that the reference `P`
    /// dereferences to will not be moved out of again; in particular, it
    /// must not be possible to obtain a `&mut P::Target` and then
    /// move out of that reference (using, for example [`mem::swap`]).
    ///
    /// For example, calling `Pin::new_unchecked` on an `&'a mut T` is unsafe because
    /// while you are able to pin it for the given lifetime `'a`, you have no control
    /// over whether it is kept pinned once `'a` ends:
    /// ```
    /// use std::mem;
    /// use std::pin::Pin;
    ///
    /// fn move_pinned_ref<T>(mut a: T, mut b: T) {
    ///     unsafe {
    ///         let p: Pin<&mut T> = Pin::new_unchecked(&mut a);
    ///         // This should mean the pointee `a` can never move again.
    ///     }
    ///     mem::swap(&mut a, &mut b);
    ///     // The address of `a` changed to `b`'s stack slot, so `a` got moved even
    ///     // though we have previously pinned it! We have violated the pinning API contract.
    /// }
    /// ```
    /// A value, once pinned, must remain pinned forever (unless its type implements `Unpin`).
    ///
    /// Similarily, calling `Pin::new_unchecked` on an `Rc<T>` is unsafe because there could be
    /// aliases to the same data that are not subject to the pinning restrictions:
    /// ```
    /// use std::rc::Rc;
    /// use std::pin::Pin;
    ///
    /// fn move_pinned_rc<T>(mut x: Rc<T>) {
    ///     let pinned = unsafe { Pin::new_unchecked(x.clone()) };
    ///     {
    ///         let p: Pin<&T> = pinned.as_ref();
    ///         // This should mean the pointee can never move again.
    ///     }
    ///     drop(pinned);
    ///     let content = Rc::get_mut(&mut x).unwrap();
    ///     // Now, if `x` was the only reference, we have a mutable reference to
    ///     // data that we pinned above, which we could use to move it as we have
    ///     // seen in the previous example. We have violated the pinning API contract.
    ///  }
    ///  ```
    ///
    /// [`mem::swap`]: ../../std/mem/fn.swap.html
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub unsafe fn new_unchecked(pointer: P) -> Pin<P> {
        Pin { pointer }
    }

    /// Gets a pinned shared reference from this pinned pointer.
    ///
    /// This is a generic method to go from `&Pin<Pointer<T>>` to `Pin<&T>`.
    /// It is safe because, as part of the contract of `Pin::new_unchecked`,
    /// the pointee cannot move after `Pin<Pointer<T>>` got created.
    /// "Malicious" implementations of `Pointer::Deref` are likewise
    /// ruled out by the contract of `Pin::new_unchecked`.
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub fn as_ref(self: &Pin<P>) -> Pin<&P::Target> {
        unsafe { Pin::new_unchecked(&*self.pointer) }
    }
}

impl<P: DerefMut> Pin<P> {
    /// Gets a pinned mutable reference from this pinned pointer.
    ///
    /// This is a generic method to go from `&mut Pin<Pointer<T>>` to `Pin<&mut T>`.
    /// It is safe because, as part of the contract of `Pin::new_unchecked`,
    /// the pointee cannot move after `Pin<Pointer<T>>` got created.
    /// "Malicious" implementations of `Pointer::DerefMut` are likewise
    /// ruled out by the contract of `Pin::new_unchecked`.
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub fn as_mut(self: &mut Pin<P>) -> Pin<&mut P::Target> {
        unsafe { Pin::new_unchecked(&mut *self.pointer) }
    }

    /// Assigns a new value to the memory behind the pinned reference.
    ///
    /// This overwrites pinned data, but that is okay: its destructor gets
    /// run before being overwritten, so no pinning guarantee is violated.
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub fn set(self: &mut Pin<P>, value: P::Target)
    where
        P::Target: Sized,
    {
        *(self.pointer) = value;
    }
}

impl<'a, T: ?Sized> Pin<&'a T> {
    /// Constructs a new pin by mapping the interior value.
    ///
    /// For example, if you  wanted to get a `Pin` of a field of something,
    /// you could use this to get access to that field in one line of code.
    /// However, there are several gotchas with these "pinning projections";
    /// see the [`pin` module] documentation for further details on that topic.
    ///
    /// # Safety
    ///
    /// This function is unsafe. You must guarantee that the data you return
    /// will not move so long as the argument value does not move (for example,
    /// because it is one of the fields of that value), and also that you do
    /// not move out of the argument you receive to the interior function.
    ///
    /// [`pin` module]: ../../std/pin/index.html#projections-and-structural-pinning
    #[stable(feature = "pin", since = "1.33.0")]
    pub unsafe fn map_unchecked<U, F>(self: Pin<&'a T>, func: F) -> Pin<&'a U> where
        F: FnOnce(&T) -> &U,
    {
        let pointer = &*self.pointer;
        let new_pointer = func(pointer);
        Pin::new_unchecked(new_pointer)
    }

    /// Gets a shared reference out of a pin.
    ///
    /// This is safe because it is not possible to move out of a shared reference.
    /// It may seem like there is an issue here with interior mutability: in fact,
    /// it *is* possible to move a `T` out of a `&RefCell<T>`. However, this is
    /// not a problem as long as there does not also exist a `Pin<&T>` pointing
    /// to the same data, and `RefCell<T>` does not let you create a pinned reference
    /// to its contents. See the discussion on ["pinning projections"] for further
    /// details.
    ///
    /// Note: `Pin` also implements `Deref` to the target, which can be used
    /// to access the inner value. However, `Deref` only provides a reference
    /// that lives for as long as the borrow of the `Pin`, not the lifetime of
    /// the `Pin` itself. This method allows turning the `Pin` into a reference
    /// with the same lifetime as the original `Pin`.
    ///
    /// ["pinning projections"]: ../../std/pin/index.html#projections-and-structural-pinning
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub fn get_ref(self: Pin<&'a T>) -> &'a T {
        self.pointer
    }
}

impl<'a, T: ?Sized> Pin<&'a mut T> {
    /// Converts this `Pin<&mut T>` into a `Pin<&T>` with the same lifetime.
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub fn into_ref(self: Pin<&'a mut T>) -> Pin<&'a T> {
        Pin { pointer: self.pointer }
    }

    /// Gets a mutable reference to the data inside of this `Pin`.
    ///
    /// This requires that the data inside this `Pin` is `Unpin`.
    ///
    /// Note: `Pin` also implements `DerefMut` to the data, which can be used
    /// to access the inner value. However, `DerefMut` only provides a reference
    /// that lives for as long as the borrow of the `Pin`, not the lifetime of
    /// the `Pin` itself. This method allows turning the `Pin` into a reference
    /// with the same lifetime as the original `Pin`.
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub fn get_mut(self: Pin<&'a mut T>) -> &'a mut T
        where T: Unpin,
    {
        self.pointer
    }

    /// Gets a mutable reference to the data inside of this `Pin`.
    ///
    /// # Safety
    ///
    /// This function is unsafe. You must guarantee that you will never move
    /// the data out of the mutable reference you receive when you call this
    /// function, so that the invariants on the `Pin` type can be upheld.
    ///
    /// If the underlying data is `Unpin`, `Pin::get_mut` should be used
    /// instead.
    #[stable(feature = "pin", since = "1.33.0")]
    #[inline(always)]
    pub unsafe fn get_unchecked_mut(self: Pin<&'a mut T>) -> &'a mut T {
        self.pointer
    }

    /// Construct a new pin by mapping the interior value.
    ///
    /// For example, if you  wanted to get a `Pin` of a field of something,
    /// you could use this to get access to that field in one line of code.
    /// However, there are several gotchas with these "pinning projections";
    /// see the [`pin` module] documentation for further details on that topic.
    ///
    /// # Safety
    ///
    /// This function is unsafe. You must guarantee that the data you return
    /// will not move so long as the argument value does not move (for example,
    /// because it is one of the fields of that value), and also that you do
    /// not move out of the argument you receive to the interior function.
    ///
    /// [`pin` module]: ../../std/pin/index.html#projections-and-structural-pinning
    #[stable(feature = "pin", since = "1.33.0")]
    pub unsafe fn map_unchecked_mut<U, F>(self: Pin<&'a mut T>, func: F) -> Pin<&'a mut U> where
        F: FnOnce(&mut T) -> &mut U,
    {
        let pointer = Pin::get_unchecked_mut(self);
        let new_pointer = func(pointer);
        Pin::new_unchecked(new_pointer)
    }
}

#[stable(feature = "pin", since = "1.33.0")]
impl<P: Deref> Deref for Pin<P> {
    type Target = P::Target;
    fn deref(&self) -> &P::Target {
        Pin::get_ref(Pin::as_ref(self))
    }
}

#[stable(feature = "pin", since = "1.33.0")]
impl<P: DerefMut> DerefMut for Pin<P>
where
    P::Target: Unpin
{
    fn deref_mut(&mut self) -> &mut P::Target {
        Pin::get_mut(Pin::as_mut(self))
    }
}

#[unstable(feature = "receiver_trait", issue = "0")]
impl<P: Receiver> Receiver for Pin<P> {}

#[stable(feature = "pin", since = "1.33.0")]
impl<P: fmt::Debug> fmt::Debug for Pin<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.pointer, f)
    }
}

#[stable(feature = "pin", since = "1.33.0")]
impl<P: fmt::Display> fmt::Display for Pin<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.pointer, f)
    }
}

#[stable(feature = "pin", since = "1.33.0")]
impl<P: fmt::Pointer> fmt::Pointer for Pin<P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.pointer, f)
    }
}

// Note: this means that any impl of `CoerceUnsized` that allows coercing from
// a type that impls `Deref<Target=impl !Unpin>` to a type that impls
// `Deref<Target=Unpin>` is unsound. Any such impl would probably be unsound
// for other reasons, though, so we just need to take care not to allow such
// impls to land in std.
#[stable(feature = "pin", since = "1.33.0")]
impl<P, U> CoerceUnsized<Pin<U>> for Pin<P>
where
    P: CoerceUnsized<U>,
{}

#[stable(feature = "pin", since = "1.33.0")]
impl<'a, P, U> DispatchFromDyn<Pin<U>> for Pin<P>
where
    P: DispatchFromDyn<U>,
{}