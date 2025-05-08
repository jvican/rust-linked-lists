pub mod fifth;
pub mod first;
pub mod fourth;
pub mod second;
pub mod sixth;
pub mod stacklist;
pub mod third;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

/// ```compile_fail
/// use rust_linked_lists::sixth::IterMut;
///
/// fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
/// ```
fn iter_mut_invariant() {}
