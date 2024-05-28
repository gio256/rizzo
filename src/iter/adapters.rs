use crate::iter::LendIter;

/// This `struct` is created by [`windows_mut`].
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct WindowsMut<T, const N: usize> {
    slice: T,
    start: usize,
}

impl<T, const N: usize> WindowsMut<T, N> {
    pub fn new(slice: T) -> Self {
        Self { slice, start: 0 }
    }
}

impl<'a, T, const N: usize> LendIter for WindowsMut<&'a mut [T], N> {
    type Item<'n> = &'n mut [T; N] where Self: 'n;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let res = self.slice[self.start..].get_mut(..N)?;
        self.start += 1;
        Some(res.try_into().unwrap())
    }
}

/// Returns a lending iterator over all contiguous windows of length `N`.
/// If the slice is shorter than `N`, the iterator returns no values.
pub fn windows_mut<T, const N: usize>(slice: &mut [T]) -> WindowsMut<&mut [T], N> {
    WindowsMut::new(slice)
}

/// This `struct` is created by [`LendIter::zip`].
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Zip<A, B> {
    a: A,
    b: B,
}

impl<A, B> Zip<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: LendIter, B: Iterator> LendIter for Zip<A, B> {
    type Item<'n> = (A::Item<'n>, B::Item) where Self: 'n;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let a = self.a.next()?;
        let b = self.b.next()?;
        Some((a, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_zip() {
        let mut xs = [0, 1, 2, 3, 4, 5];
        let ys = [6, 7, 8, 9];

        let mut iter = windows_mut::<_, 2>(&mut xs).zip(ys);
        let mut expect = [(0, 1, 6), (1, 2, 7), (2, 3, 8), (3, 4, 9)].into_iter();
        while let Some(([a, b], c)) = iter.next() {
            assert_eq!((*a, *b, c), expect.next().unwrap());
        }
        assert!(expect.next().is_none());
    }
}
