// Let's avoid Rc and RefCell for now and roll back to implementing a singly
// linked list unsafely. This time, we're going to dip our toes into raw
// pointers and unsafe in Rust, to fully understand a few layout challenges.

// This singly linked list is a variation of the stack in 'second.rs', with the
// exception that this time our list behaves like a queue, so push and pop act
// at the end of the list rather than the beginning.

use std::mem;
use std::ptr;

// This implementation uses mutable pointers in the interface, but they are
// hidden from the users given that we define them in structs. Nonetheless,
// we really don't want to expose
pub struct List<T> {
    head: Link<T>,
    // We avoid the use of mutable references because
    // tail: Option<&mut Node<T>>,
    tail: *mut Node<T>,
}

// We don't want to mix Box with mutable pointers, so we avoid:
// type Link<T> = Option<Box<Node<T>>>;
// And, instead, use the following definition:
type Link<T> = *mut Node<T>;
// Note that Option above is not even that useful when using mutable pointers,
// because we already have a null value (the null pointer)

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        unsafe {
            // We could also allocate memory manually with std::alloc::alloc
            // But that's a big footgun we generally try to avoid in Rust
            let new_tail = Box::into_raw(Box::new(Node {
                elem: elem,
                next: ptr::null_mut(),
            }));

            if !self.tail.is_null() {
                (*self.tail).next = new_tail;
            } else {
                self.head = new_tail;
            }

            self.tail = new_tail;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                None
            } else {
                let head = Box::from_raw(self.head);
                self.head = head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }

                Some(head.elem)
            }
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // Repeatedly popping
        while let Some(_) = self.pop() {}
    }
}

pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T> {
    // Given we no longer use safe pointers anywhere in the linked list
    // implementation, we preferred not to use them in the iter interfaces...
    // Ideally we'd use something like:
    // next: *mut Node<T>,
    // However, if we do that then the lifetime is not used. We could use
    // `PhantomData` to work around this... or, instead, we can seettle for
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        unsafe {
            Iter {
                next: self.head.as_ref(),
            }
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            IterMut {
                // `as_mut` type definition contains an unbounded lifetime:
                // pub unsafe fn as_mut<'a>(self) -> Option<'a mut T>
                // That's a lifetime unattached to the input, and it's nasty
                // because it's willing to pretend to be as large as specified
                // by the caller, even 'static! This is a smell but we push through
                next: self.head.as_mut(),
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        unsafe { self.head.as_ref().map(|node| &node.elem) }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe { self.head.as_mut().map(|node| &mut node.elem) }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.map(|node| {
                self.next = node.next.as_ref();
                &node.elem
            })
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.take().map(|node| {
                self.next = node.next.as_mut();
                &mut node.elem
            })
        }
    }
}

mod checks {
    fn test_arrays() {
        unsafe {
            let mut data = [0; 10];
            let slice1 = &mut data[..];
            let (slice2_at_0, slice3_at_1) = slice1.split_at_mut(1);

            let ref4_at_0 = &mut slice2_at_0[0]; // Reference to 0th element
            let ref5_at_1 = &mut slice3_at_1[0]; // Reference to 1th element
            let ptr6_at_0 = ref4_at_0 as *mut i32; // Ptr to 0th element
            let ptr7_at_1 = ref5_at_1 as *mut i32; // Ptr to 1th element

            *ptr7_at_1 += 7;
            *ptr6_at_0 += 6;
            *ref5_at_1 += 5;
            *ref4_at_0 += 4;

            // Should be [10, 12, 0, ...]
            println!("{:?}", &data[..]);
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        list.push(4);
        list.push(5);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}

#[test]
fn miri_food() {
    let mut list = List::new();

    list.push(1);
    list.push(2);
    list.push(3);

    assert!(list.pop() == Some(1));
    list.push(4);
    assert!(list.pop() == Some(2));
    list.push(5);

    assert!(list.peek() == Some(&3));
    list.push(6);
    list.peek_mut().map(|x| *x *= 10);
    assert!(list.peek() == Some(&30));
    assert!(list.pop() == Some(30));

    for elem in list.iter_mut() {
        *elem *= 100;
    }

    let mut iter = list.iter();
    assert_eq!(iter.next(), Some(&400));
    assert_eq!(iter.next(), Some(&500));
    assert_eq!(iter.next(), Some(&600));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);

    assert!(list.pop() == Some(400));
    list.peek_mut().map(|x| *x *= 10);
    assert!(list.peek() == Some(&5000));
    list.push(7);

    // Drop it on the ground and let the dtor exercise itself
}
