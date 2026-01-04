// Reminder: an enum is a value that can be of in one of the possible states
// The way it's represented at a low-level is through a tag which indicates the
// kind of type it is, plus the memory to store the largest value in the field.
//
// The problem with this list definition is that when we instantiate the enum,
// its reference will exist in the stack as such:
//
//   [Elem A, ptr] -> (Elem B, ptr) -> (Empty, *junk*)
//
// Legend
// ======
// [] = Stack
// () = Heap
//
// Note that the above memory layout occurs because:
// - The Box instance allocates BadList1 in the heap
// - The Cons definition takes 96 bits (32 + 64 bits due to the pointer)
//
// As such, when BadList1 is of the Empty variant and should take 1 bit of
// space, it takes 96 bits out  of which most of them are junk and unused.
// This is due to how enums are typically encoded, because we can change the
// enum type any time we want, changing an empty instance to a non-empty one.
//
// The non-uniform memory layout above (part in the heap, part in the stack) is
// problematic because it imposes more work on us when we manipulate it. For
// example, when splitting a list we have to do double work.
pub enum BadList1 {
    Nil,
    Cons(i32, Box<BadList1>),
}

// If we wanted to naively avoid the previous issue, we could add a new enum
// type ElemThenEmpty. However, this new option is much worse for two main reasons:
//
// 1. First, we're exposing implementation details on the user public API. Users
// will be able to see that extra subclass.
//
// 2. Second, we lose the null-pointer optimization. Previously, Rust could
// represent the empty case without an extra tag field because a zero pointer
// isn't a valid state for the `Cons` type.  With the newly added enum type,
// that optimization can't be used anymore.
pub enum BadList2 {
    Nil,
    ElemThenEmpty(i32),
    ElemThenNonEmpty(i32, Box<BadList2>),
}

// So, next up: how else can we model this linked list? We try to instead use a
// C-like struct model. For that, we can use Rust structs. In Rust structs, the
// layout information is opaque to the consumer. We start by separating the
// definition of the list from that of the node.

// We start with the list definition, noting that this new version:
// - Has only two cases, so the `Empty` case is efficiently represented
// - The tail of the list doesn't allocate extra junk
// - All elements are uniformly allocated in the heap!
pub enum List1 {
    Empty,
    More(Box<Node1>),
}

// Now, in theorty, we wish to mark this as non-public as it's a good Rust
// design pattern. However, Rust doesn't allow us. `List1` is a
// publically-accessible enum and it'd otherwise refer to a private type `Node1`
// in its public structure (a permission violation).
// -- (Note that we marked it as public below so it compiles)
pub struct Node1 {
    elem: i32,
    next: List1,
}

// As we don't want to expose implementation details from this package, we lean
// in on defining the list as a struct instead (which doesn't have the same
// limitation as enum), and create an extra enum type `Link` that takes the
// place of our previous List1. This extra enum type can be kept internal.
//
// An additional benefit of this struct definition is that it's a zero-cost
// abstraction. Rust can skip the struct representation for one-field structs.
pub struct List {
    head: Link,
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    // Note that we connect with Link and not List!
    // No benefit in connecting with List again...
    next: Link,
}

impl List {
    // Self is an alias of the type I wrote next to `impl`: `List``
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    pub fn push(&mut self, elem: i32) {
        let next = std::mem::replace(&mut self.head, Link::Empty);
        let new_node = Node { elem, next };

        // Huh, we don't need a replace anymore, this makes sense!
        // std::mem::replace(&mut self.head, Link::More(Box::new(new_node)));
        self.head = Link::More(Box::new(new_node));
    }

    pub fn pop(&mut self) -> Option<i32> {
        let curr = std::mem::replace(&mut self.head, Link::Empty);
        match curr {
            Link::Empty => Option::None,
            Link::More(node) => {
                self.head = node.next;
                Option::Some(node.elem)
            }
        }
    }
}

impl Drop for List {
    fn drop(&mut self) {
        let mut curr = std::mem::replace(&mut self.head, Link::Empty);
        // We prefer the use of this pattern rather than reusing pop because pop
        // moves the values from the heap to the stack, and in cases where T can be
        // a large object with a drop implementation that would be inefficient. As
        // this implementation manipulates links (Box<Node>), moving the reference is
        // not copying the data around, and when it goes out of scope we drop the value
        while let Link::More(mut boxed_node) = curr {
            curr = std::mem::replace(&mut boxed_node.next, Link::Empty);
            // After this point, boxed_node goes out of scope and it's freed
            // We replaced the next boxed_node with Link::Empty so no
            // unbounded recursion happens in drop
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut l = List::new();

        assert_eq!(l.pop(), None);

        l.push(1);
        l.push(2);
        l.push(3);

        assert_eq!(l.pop(), Some(3));
        assert_eq!(l.pop(), Some(2));

        l.push(4);
        l.push(5);

        assert_eq!(l.pop(), Some(5));
        assert_eq!(l.pop(), Some(4));

        assert_eq!(l.pop(), Some(1));
        assert_eq!(l.pop(), None);
    }
}
