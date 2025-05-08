# Personal Learnings about Rust Programming

Mostly a series of one-liners of learnings written up when following and coding
the linked list tutorial in Rust. A list of takeaways to remember in the future.

- `Box` is how all stack allocations occur in Rust.
- Borrows occur based on the `&` and `&mut` qualifiers.
- Borrows mostly show up in method, variable, and field signatures.
- Deref coercion is the process of inserting \* to make your code typecheck (e.g. field access).
- Manipulating borrows for generic types like `Option` requires `as_ref`, `as_mut`.
- The turbofish annotation `::<>` allows the user to specify
- For example, To turn an `&Option<T>` to `Option<&T>`, we use `as_ref`.
- Use `as_deref` to go from a Box<T> to T, equivalent to `&**node`, which often Rust does automatically.
- Most borrows happen implicitly when using API methods on data structures.
- Lifetimes are just like type parameters, bound to a specific borrow `&'a mut`.
- Lifetimes are many times inferred by Rust (lifetime ellision), they're always present.
- A lifetime is simply the name of a region (~block, scope) of somewhere in the program.
- Lifetimes are explicitly defined in APIs because Rust doesn't know how data will flow.
- A lifetime for a N-ary function assumes all N params have independent lifetimes.
- A lifetime of a method is automatically inferred to the `Self` lifetime regardless of its parameters.
- A static lifetime corresponds to
- A static lifetime `&'static T` outlives all other lifetimes, so it's a subtype of any `&'a T`
- Use std::mem::replace to swap the value of a struct (none of the values are dropped)
- Implementing capabilities is done through typeclass-like interfaces (Iter, IterMut).
- The structure of `enum`s is transparent to consumers, while that of `struct` is not.
- There's no promise of memory alignment and field order in enums and struct by default.
- Rust defines most new data structures through the use of structs, enums, and type defns.
- Public API methods for specific type definitions occur in an `impl` companion of the same name.
- One can use `Self` to refer to defining type within this `impl` companion.
- Rust allows accessing struct fields through definition-specific field indexes `self.0`.
- One defines modules through `mod` and imports externally defined modules through `use`.

- Rust conceptually handles reborrows by maintaining a "borrow stack"
- Only the one on the top of the stack is "live" (has exclusive access)
- When you access a lower one it becomes "live" and the ones above it get popped
- You're not allowed to use pointers that have been popped from the borrow stack
- The borrow checker ensures safe code obeys this
- Miri theoretically checks that raw pointers obey this at runtime

- Rust doesn't distinguish between refs to different array indices, but one can use `split_at_mut`
- Once a shared reference is on the "borrow stack", everything pushed on top only can only read
- `&*ptr` is a way to writing down the memory address of the thing the ptr points to (without loading it)
- `&**ptr` is in fact loading the value with the first `*`
- Casting an immutable reference (`&T`) directly to `*mut T` and then writing to it is UB (Workaround: derive `*mut T` from `&mutT`)
- An associated function is akin to a static method in the "companion": `Box::into_raw(b)`, not `b.into_raw()`
- Use `Box::into_raw` to convert a box to a `*mut T` (thus becoming the owner)
- Use `Box::from_raw` to do the opposite operation of `Box::into_raw()`, useful for cleanly dropping

- Critical rules for collections: Variance, Drop Check, NonNull, isize::MAX Allocation, Zero-Sized Types
- Lifetime subtyping isn't always safe: e.g. can use mem::swap with mutable refs and get dangling pointers
- `*mut T` is always invariant to avoid cases where covariance would be terribly dangerous
- `*const T` is always covariant: a `List<&T>` and `List<*const T>` should be equivalent
- `NonNull` casts `*const T` to `*mut T` to provide users with a covariant equivalent
- Use `PhantomData` to control variance and indicate a type "owns" another even though it doesn't store it
- The Drop Checker in Rust analyzes `Drop` implementation to ensure they are safe borrow-wise
- A simple integrity check can cause an exploitable memory safety bug because it can panic!
- Panics are 'unwinding' (immediately return); they can be caught and prevent destructors from running
- Be extremely vigilant with panics when using unsafe pointers: can products bugs like 'use-after-free''''
- Even arithmetic operations can cause panics (memory-safety issues) like underflows
- You can't rely on destructors in code that you don't control to run (others can use std::mem::forget)
- In some cases, you can actually use destructors for panic safety (see BinaryHeap::sift_up case study)
- Handy iterators to implement: `FusedIterator`, `ExactSizedIterator`, `DoubleEndedIterator`
- Unlike other languages, Rust data structures usually implement Clone, Copy, Extend
- The Debug trait provides a way to print the structure of a type (an effectful Show typeclass)
- Implementing `IntoIterator` for T will make it work with for-loops out-of-the-box
- `Send` tells Rust your type is safe to send to another thread
- `Sync` tells Rust your type is safe to share between threads
- When collections or types don't use fancy interior mut tricks, they're safe to make Send and Sync
- Shared and mutables pointers in Rust opt out of `Send` and `Sync` _just_ to be safe
- You can opt out of certain traits by default by doing `impl !Typeclass for YourType {}`
- You can opt into certain traits with `unsafe impl<T: Send> Send for LinkedList<T> {}`
- Write code in the doccomment of a _PUBLIC_ method to check compilation/non-compilation of your API
- Be vigilant about default properties your types opt-in automatically (e.g. check iterator covariance)
- By-ref iterators (with `type Item = &'a T`) borrow from the collection
- By-mutable-ref iterators (with `type Item = &'a mut T`) borrow mutable refs from the collection
- By-mutable-ref iterators can never go backwards and yield an element again (two &mut's to same value)
- By-ref iterators can't have any public methods that would modify the underlying collection
- Cursors represents a position in a sequence that you can move around and make edits at
- Lifetime elision implies that the return type's lifetime will be the same as that from the fn's param
- Use `as_deref()` to convert from `Option<T>` or `&Option<T>` to `Option<&T>
- Use `let mut` when you want to create mutable borrows to local variables

QUESTIONS

- What's the difference between IntoIter and Iter?
- What's the difference between by-value and by-ref semantics in Rust?
- What are the recommended ways of doing property testing with Rust?
- What does `auto trait` do?
- What does `std::io::Cursor` do?
- In what cases are lifetimes static?
- What are the most common usages of macros (in std and popular libraries)?
- What are notorious Rust examples or APIs to do unsafe-like functionality but with safe abstractions?
- How could we keep expressions that could panic but handle them in such a way that all destructors run?

Raw pointers are basically C's pointers. They have no inherent aliasing rules. They have no lifetimes. They can be null. They can be misaligned. They can be dangling. They can point to uninitialized memory. They can be cast to and from integers. They can be cast to point to a different type. Mutability? Cast it. Pretty much everything goes, and that means pretty much anything can go wrong.

```rust
fn opaque_read(val: &i32) {
    println!("{}", val);
}

unsafe {
    let mut data = 10;
    let mref1 = &mut data;
    let ptr2 = mref1 as *mut i32;
    let sref3 = &*mref1;

    // This would trigger a compilation error because we can't derive a mut pointer from a shared reference
    // let ptr4 = sref3 as *mut i32;

    // However... the following will compile (even though miri will detect the UB error)
    // > error: Undefined Behavior: no item granting write access to tag <1621> found in borrow stack
    let ptr4 = sref3 as *const i32 as *mut i32;

    *ptr4 += 4;
    opaque_read(sref3);
    *ptr2 += 2;
    *mref1 += 1;

    opaque_read(&data);
}
```

Interior mutability. `UnsafeCell` is a core Rust primitive that opts out of the immutability guarantee for &T, so reads cannot assume the data hasn't changed in memory, and may point to data that is being mutated. This is what interior mutability is.

```rust
use std::cell::Cell;

fn opaque_read(val: &i32) {
    println!("{}", val);
}

unsafe {
    let mut data = UnsafeCell::new(10);
    let mref1 = &mut data;              // Mutable ref to the *outside*
    let ptr2 = mref1.get();             // Get a raw pointer to the insides
    let sref3 = &*mref1;                // Get a shared ref to the *outside*

    *ptr2 += 2;                         // Mutate with the raw pointer
    opaque_read(&*sref3.get());         // Read from the shared ref
    *sref3.get() += 3;                  // Write through the shared ref
    *mref1.get() += 1;                  // Mutate with the mutable ref

    println!("{}", *data.get());
}
```

What is the diff between *mut i32 and *const i32?

One is a shared mutable pointer, the other one is a shared pointer.
A shared pointer \*const i32 is different from &i32.
