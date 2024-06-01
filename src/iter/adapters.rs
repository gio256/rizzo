use crate::iter::LendIter;

/// Returns a lending iterator over all contiguous windows of length `N`.
/// If the slice is shorter than `N`, the iterator returns no values.
pub(crate) fn windows_mut<T, const N: usize>(slice: &mut [T]) -> Windows<&mut [T], N> {
    Windows::new(slice)
}

/// This `struct` is created by [`windows_mut`].
#[derive(Clone, Copy, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct Windows<T, const N: usize> {
    slice: T,
    head: usize,
}

impl<T, const N: usize> Windows<T, N> {
    pub(crate) fn new(slice: T) -> Self {
        const { assert!(N != 0, "window size must be nonzero") }
        Self { slice, head: 0 }
    }
}

impl<'a, T, const N: usize> LendIter for Windows<&'a mut [T], N> {
    type Item<'n> = &'n mut [T; N] where Self: 'n;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let res = self.slice.get_mut(self.head..)?.get_mut(..N)?;
        self.head += 1;
        Some(res.try_into().unwrap())
    }
}

/// This `struct` is created by [`LendIter::zip_iter`].
#[derive(Clone, Copy, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct Zip<A, B> {
    a: A,
    b: B,
}

impl<A, B> Zip<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: LendIter, B: LendIter> LendIter for Zip<A, B> {
    type Item<'n> = (A::Item<'n>, B::Item<'n>) where Self: 'n;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let a = self.a.next()?;
        let b = self.b.next()?;
        Some((a, b))
    }
}

/// Wraps an `Iterator` into an implementer of [`LendIter`].
#[derive(Clone, Copy, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct Lend<I> {
    iter: I,
}

impl<I> Lend<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }

    pub(crate) fn from_iter<J: IntoIterator<IntoIter = I>>(it: J) -> Self {
        Self::new(it.into_iter())
    }
}

impl<I: Iterator> LendIter for Lend<I> {
    type Item<'n> = I::Item where I: 'n;

    fn next(&mut self) -> Option<I::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_zip() {
        let mut xs = [0, 1, 2, 3, 4, 5];
        let ys = [6, 7, 8, 9];

        let mut iter = windows_mut::<_, 2>(&mut xs).zip_iter(ys);
        let mut expect = [(0, 1, 6), (1, 2, 7), (2, 3, 8), (3, 4, 9)].into_iter();
        while let Some(([a, b], c)) = iter.next() {
            assert_eq!((*a, *b, c), expect.next().unwrap());
        }
        assert!(expect.next().is_none());
    }
}
