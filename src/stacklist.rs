pub struct List<'a, T> {
    pub data: T,
    pub prev: Option<&'a List<'a, T>>,
}

pub struct Iter<'a, T> {
    next: Option<&'a List<'a, T>>,
}

impl<'a, T> List<'a, T> {
    pub fn push<U>(
        prev: Option<&'a List<'a, T>>,
        data: T,
        callback: impl FnOnce(&List<'a, T>) -> U,
    ) -> U {
        let list = List { data, prev };
        callback(&list)
    }

    pub fn iter(&'a self) -> Iter<'a, T> {
        Iter { next: Some(self) }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // self.next = node.prev.as_ref().map(|prev| &**prev)
            self.next = node.prev.as_deref();
            &node.data
        })
    }
}

// Example of usage:
// List::push(None, 3, |list| {
//     println!("{}", list.data);
//     List::push(Some(list), 5, |list| {
//         println!("{}", list.data);
//         List::push(Some(list), 13, |list| {
//             println!("{}", list.data);
//         })
//     })
// })

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn elegance() {
        List::push(None, 3, |list| {
            assert_eq!(list.iter().copied().sum::<i32>(), 3);
            List::push(Some(list), 5, |list| {
                assert_eq!(list.iter().copied().sum::<i32>(), 5 + 3);
                List::push(Some(list), 13, |list| {
                    assert_eq!(list.iter().copied().sum::<i32>(), 13 + 5 + 3);
                })
            })
        })
    }

    fn example1() {
        let mut x = Box::new(42);
        let r = &x;

        if rand::random::<f64>() > 0.5 {
            *x = 24;

            // Doing this at this position would've caused a compiler error in
            // the previous line because the borrow would be invalid:
            // println!("{}", r);
        } else {
            println!("{}", r);
        }
    }

    fn example2() {
        let mut x = Box::new(42);
        let mut z = &x;
        for i in 1..100 {
            println!("{}", z);

            // When we overwrite x, then we're invalidating the borrow on x made
            // when initializing z If we didn't re-borrow the new x value in z,
            // Rust would fail with a "cannot assign to x because it is already
            // borrowed". Which kind of proves the importance of
            x = Box::new(i);
            z = &x;
        }
    }

    fn example3() {
        struct MutStr<'a, 'b> {
            s: &'a mut &'b str,
        }

        let mut s = "hello";
        *MutStr { s: &mut s }.s = "world";
        println!("{}", s);
    }
}

#[test]
fn cell() {
    use std::cell::Cell;

    List::push(None, Cell::new(3), |list| {
        List::push(Some(list), Cell::new(5), |list| {
            List::push(Some(list), Cell::new(13), |list| {
                // Multiply every value in the list by 10
                for val in list.iter() {
                    val.set(val.get() * 10)
                }

                let mut vals = list.iter();
                assert_eq!(vals.next().unwrap().get(), 130);
                assert_eq!(vals.next().unwrap().get(), 50);
                assert_eq!(vals.next().unwrap().get(), 30);
                assert_eq!(vals.next(), None);
                assert_eq!(vals.next(), None);
            })
        })
    })
}
